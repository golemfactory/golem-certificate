use golem_certificate::schemas::{certificate::key_usage::KeyUsage, permissions::Permissions};
use url::Url;

pub struct X {}

struct Tree {
    entries: Vec<TreeEntry>,
}

enum TreeValue {
    KeyUsage(Vec<KeyUsage>),
    Object(Tree),
    Permissions(Permissions),
    Text(String),
}

struct TreeEntry {
    name: String,
    value: TreeValue,
}

struct PermissionsEditor {
    urls: Vec<Url>,
    permissions: Permissions,
}
