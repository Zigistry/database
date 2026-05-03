/*!
 * SPDX-License-Identifier: AGPL-3.0-only WITH LicenseRef-Zigistry-Database-Permission
 *
 * Copyright (c) 2025 Rohan Vashisht
 *
 * This software is licensed under the GNU Affero General Public License v3.0.
 * The database content under `zigistry/database` is subject to additional terms.
 *
 *      ______       _     _
 *     |__  (_) __ _(_)___| |_ _ __ _   _
 *       / /| |/ _` | / __| __| '__| | | |
 *      / /_| | (_| | \__ \ |_| |  | |_| |
 *     /____|_|\__, |_|___/\__|_|   \__, |
 *             |___/                |___/
 *
 *
 * See LICENSE and LICENSE-ADDITIONAL in the project root directory for full details.
 */

pub mod bzz_stuff;
pub mod codeberg;
pub mod constants;
pub mod custom_types;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub mod database;
pub mod github;

use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref GITHUB_KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    pub static ref CODEBERG_KEY: String =
        "token ".to_string() + &env::var("CB_API_KEY").expect("CB_API_KEY not set");
    static ref PYTHON_SETUP: () = {
        Python::attach(|py| {
            py.run(
                c"
from nltk.corpus import stopwords
from keybert import KeyBERT
import nltk

try:
    stopwords.words('english')
except:
    nltk.download('stopwords')

kw_model = KeyBERT()

stop_words = stopwords.words('english')

ZIG_STOPWORDS = [
    'package', 'packages', 'program', 'programs', 'project', 'projects',
    'repo', 'repository', 'source', 'code', 'coding', 'implementation',
    'example', 'examples', 'demo', 'sample', 'samples',
    'template', 'boilerplate', 'starter', 'scaffold',
    'utilities', 'utility', 'toolkit', 'tools', 'tool',
    'lib', 'libs', 'library', 'libraries',

    'zig', 'ziglang', 'zig-package', 'std', 'stdlib',
    'standard library', 'comptime', 'build.zig', 'zigmod',
    'install', 'installation', 'setup', 'configure', 'configuration',
    'config', 'usage', 'howto', 'guide', 'tutorial', 'walkthrough',
    'docs', 'documentation', 'readme', 'changelog', 'release',
    'releases', 'version', 'v1', 'v2', 'v3',
    'dependency', 'dependencies', 'dep', 'deps', 'registry', 'index',
    'manifest', 'lockfile',

    'github', 'gitlab', 'codeberg', 'bitbucket', 'git',
    'commit', 'commits', 'branch', 'fork',
    'pull request', 'pr', 'issue', 'issues',
    'tags', 'tag',
    'algorithm', 'data structure', 'utils', 'helper', 'helpers',
    'core', 'base', 'main', 'app', 'application',
    'service', 'server', 'client',
    'backend', 'frontend', 'api', 'sdk',
    'the', 'a', 'an', 'and', 'or', 'of', 'for', 'in', 'on',
    'with', 'by', 'to', 'from', 'using', 'written',
    'cross platform', 'cross-platform', 'portable',
    'high performance', 'fast', 'lightweight', 'minimal',
    'simple', 'easy', 'best', 'awesome', 'collection', 'list', 'curated'
]

stop_words.extend(ZIG_STOPWORDS)
",
                None,
                None,
            )
            .unwrap();
        });
    };
}

pub fn keyword_extraction(doc: &str) -> PyResult<String> {
    lazy_static::initialize(&PYTHON_SETUP);

    Python::attach(|py| {
        let locals = PyDict::new(py);

        locals.set_item("doc", doc)?;

        py.run(
            c"
top_n = max(1, abs(int(len(doc.split()) * 15 / 100)))

keywords = kw_model.extract_keywords(
    doc,
    keyphrase_ngram_range=(1, 1),
    stop_words=stop_words,
    top_n=top_n
)

all_list_of_output = [i[0] for i in keywords[:200]]

processed_keywords = ' '.join(all_list_of_output)
",
            None,
            Some(&locals),
        )?;

        locals.get_item("processed_keywords")?.unwrap().extract()
    })
}
