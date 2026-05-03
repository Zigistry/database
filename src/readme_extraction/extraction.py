
from nltk.corpus import stopwords
from keybert import KeyBERT
import nltk
nltk.download('stopwords')

doc = os.environ["DOC"] # Someone might inject python script directly
# hence securing it using environ variable, not using strings from
# rust directly.

kw_model = KeyBERT()
stop_words = stopwords.words('english')

ZIG_STOPWORDS = [
    "package", "packages", "program", "programs", "project", "projects",
    "repo", "repository", "source", "code", "coding", "implementation",
    "example", "examples", "demo", "sample", "samples",
    "template", "boilerplate", "starter", "scaffold",
    "utilities", "utility", "toolkit", "tools", "tool",
    "lib", "libs", "library", "libraries",

    "zig", "ziglang", "zig-package", "std", "stdlib",
    "standard library", "comptime", "build.zig", "zigmod",
    "install", "installation", "setup", "configure", "configuration",
    "config", "usage", "howto", "guide", "tutorial", "walkthrough",
    "docs", "documentation", "readme", "changelog", "release",
    "releases", "version", "v1", "v2", "v3",
    "dependency", "dependencies", "dep", "deps", "registry", "index", "manifest", "lockfile",

    "github", "gitlab", "codeberg", "bitbucket", "git",
    "commit", "commits", "branch", "fork",
    "pull request", "pr", "issue", "issues",
    "tags", "tag",
    "algorithm", "data structure", "utils", "helper", "helpers",
    "core", "base", "main", "app", "application",
    "service", "server", "client",
    "backend", "frontend", "api", "sdk",
    "the", "a", "an", "and", "or", "of", "for", "in", "on",
    "with", "by", "to", "from", "using", "written",
    "cross platform", "cross-platform", "portable",
    "high performance", "fast", "lightweight", "minimal",
    "simple", "easy", "best", "awesome", "collection", "list", "curated"
]

top_n = abs(int(len(doc.split()) * 15/100)) # Only 15 % of the words as the keywords.

stop_words.extend(ZIG_STOPWORDS)


keywords = kw_model.extract_keywords(doc, keyphrase_ngram_range=(1, 1), stop_words=stop_words, top_n=top_n)

all_list_of_output = [i[0] for i in keywords[:200]]

processed_keywords = " ".join(all_list_of_output)

print(processed_keywords)

