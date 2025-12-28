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
        }
    }
}
