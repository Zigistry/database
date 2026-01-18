use std::collections::HashSet;

use regex::Regex;

pub async fn calculate_dependents() {
    // I will now find the gh/thingy/thingy pattern
    // Ex: github.com/zigzap/zap
    let gh_cb_pattern = Regex::new(
        r"(?i)\b(?:https?://)?(?:www\.)?(github\.com|codeberg\.org)/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/?\b"
    ).unwrap();
    let packages_keys = db.packages.keys().cloned().collect::<Vec<String>>();
    for i in packages_keys {
        let value = &db.packages[&i];
        let mut v: HashSet<String> = HashSet::new();
        for j in &value.default_branch_information.dependencies {
            v.insert(j.url.clone());
        }
        for k in &value.releases {
            for l in &k.1.dependencies {
                v.insert(l.url.clone());
            }
        }
        eprintln!("Reached Here!");
        for m in v {
            if let Some(c) = gh_cb_pattern.captures(&m) {
                let provider = c.get(1).map(|p| p.as_str());
                let owner = c.get(2).map(|p| p.as_str());
                let repo = c.get(3).map(|p| p.as_str());
                if let (Some(provider), Some(owner), Some(repo)) = (provider, owner, repo) {
                    let key = match provider {
                        "github.com" => format!("gh/{owner}/{repo}"),
                        "codeberg.org" => format!("cb/{owner}/{repo}"),
                        _ => continue,
                    }
                    .to_lowercase();
                    if let Some(p) = db.packages.get_mut(&key) {
                        p.dependents.push(i.to_lowercase());
                    }
                }
            }
        }
    }
}
