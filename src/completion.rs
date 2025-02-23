use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

static LAST_TAB_TIME: AtomicU64 = AtomicU64::new(0);
static TAB_PRESSED: AtomicBool = AtomicBool::new(false);

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

    fn insert(&mut self, word: String) {
        let mut current = self;
        for ch in word.chars() {
            current = current.children.entry(ch).or_insert(TrieNode::new());
        }
        current.is_end = true;
        current.word = word;
    }

    fn find_prefix(&self, prefix: &str) -> Vec<String> {
        let mut current = self;
        let mut results = Vec::new();

        for ch in prefix.chars() {
            if let Some(node) = current.children.get(&ch) {
                current = node;
            } else {
                return results;
            }
        }

        Self::collect_words(current, &mut results);
        results
    }

    fn collect_words(node: &TrieNode, results: &mut Vec<String>) {
        if node.is_end {
            results.push(node.word.clone());
        }

        for child in node.children.values() {
            Self::collect_words(child, results);
        }
    }

    fn find_common_prefix(&self, prefix: &str) -> Option<String> {
        let mut matches = self.find_prefix(prefix);
        if matches.is_empty() {
            return None;
        }

        if matches.len() == 1 {
            return Some(matches[0].clone() + " ");
        }

        matches.sort();
        let mut common_prefix = matches[0].clone();
        for name in &matches[1..] {
            while !name.starts_with(&common_prefix) {
                common_prefix.pop();
            }
        }

        if common_prefix.len() > prefix.len() {
            Some(common_prefix)
        } else {
            let now = Instant::now().elapsed().as_millis() as u64;
            let last_tab = LAST_TAB_TIME.load(Ordering::Relaxed);

            if now - last_tab < 500 {
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

pub struct CompletionEngine {
    builtin_commands: HashSet<String>,
    trie: Arc<RwLock<TrieNode>>,
}

impl CompletionEngine {
    pub fn new(builtins: HashSet<String>) -> Self {
        let engine = Self {
            builtin_commands: builtins,
            trie: Arc::new(RwLock::new(TrieNode::new())),
        };
        engine.refresh_cache();
        engine
    }

    pub fn refresh_cache(&self) {
        let mut trie = self.trie.write().unwrap();

        for cmd in &self.builtin_commands {
            trie.insert(cmd.clone());
        }

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

#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct RustylineHelper {
    completion_engine: CompletionEngine,
}

impl RustylineHelper {
    pub fn new(builtins: HashSet<String>) -> Self {
        Self {
            completion_engine: CompletionEngine::new(builtins),
        }
    }
}

impl rustyline::completion::Completer for RustylineHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (word_start, word) = line[..pos]
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map(|(i, _)| (i + 1, &line[i + 1..pos]))
            .unwrap_or((0, &line[..pos]));

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

