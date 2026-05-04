from nltk.corpus import stopwords
import nltk
from keybert import KeyBERT

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



kw_model = None

def main():
    nltk.download('stopwords')
    global kw_model
    kw_model = KeyBERT()

    stop_words = stopwords.words('english')
    stop_words.extend(ZIG_STOPWORDS)


if __name__ == "__main__":
    main()
