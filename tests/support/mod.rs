use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct TestProject {
    pub root: PathBuf,
}

static NEXT_PROJECT_ID: AtomicU64 = AtomicU64::new(0);

impl TestProject {
    pub fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock should be after the Unix epoch")
            .as_nanos();
        let project_id = NEXT_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "storymesh-test-{}-{unique}-{project_id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the test project should be created");
        Self { root }
    }

    pub fn add(&self, relative: &str) {
        self.add_with_contents(relative, "// test fixture\n");
    }

    pub fn add_with_contents(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the test parent should be created");
        }
        fs::write(path, contents).expect("the test file should be written");
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
