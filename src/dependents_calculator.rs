use crate::db;
use regex::Regex;

pub fn calculate_dependents() -> void {
    for i in db!().packages {
        let mut vec_of_all_urls_of_this_package = Vec::new();
        for j in i.1.default_branch_information.dependencies {
            vec_of_all_urls_of_this_package.push(j.url);
        }
        for k in i.1.releases {
            for l in k.1.dependencies {
                vec_of_all_urls_of_this_package.push(l.url);
            }
        }
        for m in vec_of_all_urls_of_this_package {
            // I will now find the gh/thingy/thingy pattern
            // Ex: github.com/zigzap/zap
            let gh_cb_pattern = Regex::new(
                r"(?i)\b(?:https?://)?(?:www\.)?(github\.com|codeberg\.org)/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/?\b"
            ).unwrap();
            let captures = gh_cb_pattern.captures(m);
            if let Some(c) = captures {
                let provider = c.get(1).map(|p| p.as_str());
                let owner = c.get(2).map(|p| p.as_str());
                let repo = c.get(3).map(|p| p.as_str());
                if let (Some(provider), Some(owner), Some(repo)) = (provider, owner, repo) {
                    if provider == "github.com" {
                        let key = format!("gh/{provider}/{repo}");
                    }
                } else {
                    continue;
                }
            }
        }
    }
}
