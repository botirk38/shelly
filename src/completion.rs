use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Track last tab press time for double-tab detection
static LAST_TAB_TIME: AtomicU64 = AtomicU64::new(0);
/// Track whether tab was pressed recently
static TAB_PRESSED: AtomicBool = AtomicBool::new(false);

/// Trie (prefix tree) node for efficient command completion
///
/// Stores commands in a tree structure where each node represents a character.
/// This enables fast prefix matching for tab completion.
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end: bool,
    word: String,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_end: false,
            word: String::new(),
        }
    }

    /// Insert a word into the trie
    fn insert(&mut self, word: String) {
        let mut current = self;
        for ch in word.chars() {
            current = current.children.entry(ch).or_insert(TrieNode::new());
        }
        current.is_end = true;
        current.word = word;
    }

    /// Find all words with the given prefix
    fn find_prefix(&self, prefix: &str) -> Vec<String> {
        let mut current = self;
        let mut results = Vec::new();

        // Navigate to the prefix node
        for ch in prefix.chars() {
            if let Some(node) = current.children.get(&ch) {
                current = node;
            } else {
                return results;
            }
        }

        // Collect all words under this prefix
        Self::collect_words(current, &mut results);
        results
    }

    /// Recursively collect all complete words from this node
    fn collect_words(node: &TrieNode, results: &mut Vec<String>) {
        if node.is_end {
            results.push(node.word.clone());
        }

        for child in node.children.values() {
            Self::collect_words(child, results);
        }
    }

    /// Find the longest common prefix for completion
    ///
    /// Returns:
    /// - If single match: the complete word with a trailing space
    /// - If multiple matches with common prefix longer than input: the common prefix
    /// - If double-tab (< 500ms): display all matches and return None
    /// - Otherwise: return None
    fn find_common_prefix(&self, prefix: &str) -> Option<String> {
        let mut matches = self.find_prefix(prefix);
        if matches.is_empty() {
            return None;
        }

        // Single match: complete with space
        if matches.len() == 1 {
            return Some(matches[0].clone() + " ");
        }

        // Find longest common prefix among all matches
        matches.sort();
        let mut common_prefix = matches[0].clone();
        for name in &matches[1..] {
            while !name.starts_with(&common_prefix) {
                common_prefix.pop();
            }
        }

        // If we can extend the prefix, do so
        if common_prefix.len() > prefix.len() {
            Some(common_prefix)
        } else {
            // Handle double-tab: show all matches if pressed within 500ms
            let now = Instant::now().elapsed().as_millis() as u64;
            let last_tab = LAST_TAB_TIME.load(Ordering::Relaxed);

            if now - last_tab < 500 {
                // Double-tab detected: show all matches
                println!("\n{}", matches.join("  "));
                print!("$ {}", prefix);
                let _ = std::io::stdout().flush();
                TAB_PRESSED.store(false, Ordering::Relaxed);
            } else {
                TAB_PRESSED.store(true, Ordering::Relaxed);
            }

            LAST_TAB_TIME.store(now, Ordering::Relaxed);
            None
        }
    }
}

/// Engine that provides command completion using a Trie for efficiency
///
/// Caches all available commands (built-ins + PATH executables) in a Trie
/// for fast prefix-based completion.
pub struct CompletionEngine {
    builtin_commands: HashSet<String>,
    trie: Arc<RwLock<TrieNode>>,
}

impl CompletionEngine {
    /// Create a new completion engine with the given built-in commands
    pub fn new(builtins: HashSet<String>) -> Self {
        let engine = Self {
            builtin_commands: builtins,
            trie: Arc::new(RwLock::new(TrieNode::new())),
        };
        engine.refresh_cache();
        engine
    }

    /// Refresh the completion cache by rebuilding the Trie
    ///
    /// Scans all directories in PATH and inserts all executable names
    /// along with built-in commands into the Trie.
    pub fn refresh_cache(&self) {
        let mut trie = self.trie.write().unwrap();

        // Add built-in commands
        for cmd in &self.builtin_commands {
            trie.insert(cmd.clone());
        }

        // Add executables from PATH
        if let Some(paths) = env::var_os("PATH") {
            for dir in env::split_paths(&paths) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.filter_map(Result::ok) {
                        if let Ok(name) = entry.file_name().into_string() {
                            trie.insert(name);
                        }
                    }
                }
            }
        }
    }
}

/// Rustyline helper that integrates with the completion engine
///
/// Implements the Completer trait to provide tab completion for commands.
/// Also derives Helper, Hinter, Highlighter, and Validator for full
/// rustyline integration.
#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct RustylineHelper {
    completion_engine: CompletionEngine,
}

impl RustylineHelper {
    /// Create a new helper with the given built-in commands
    pub fn new(builtins: HashSet<String>) -> Self {
        Self {
            completion_engine: CompletionEngine::new(builtins),
        }
    }
}

impl rustyline::completion::Completer for RustylineHelper {
    type Candidate = String;

    /// Provide completion candidates for the word at the cursor position
    ///
    /// Extracts the word being typed, searches the Trie for matches,
    /// and returns the completion suggestion.
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Find the start of the current word (after last whitespace)
        let (word_start, word) = line[..pos]
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map(|(i, _)| (i + 1, &line[i + 1..pos]))
            .unwrap_or((0, &line[..pos]));

        // Get completion from the Trie
        if let Some(completion) = self
            .completion_engine
            .trie
            .read()
            .unwrap()
            .find_common_prefix(word)
        {
            Ok((word_start, vec![completion]))
        } else {
            Ok((word_start, vec![]))
        }
    }
}
