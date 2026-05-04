from nltk.corpus import stopwords
import nltk
from keybert import KeyBERT
from fastapi import FastAPI
from contextlib import asynccontextmanager


ZIG_STOPWORDS = [
    "package",
    "packages",
    "program",
    "programs",
    "project",
    "projects",
    "repo",
    "repository",
    "source",
    "code",
    "coding",
    "implementation",
    "example",
    "examples",
    "demo",
    "sample",
    "samples",
    "template",
    "boilerplate",
    "starter",
    "scaffold",
    "utilities",
    "utility",
    "toolkit",
    "tools",
    "tool",
    "lib",
    "libs",
    "library",
    "libraries",
    "zig",
    "ziglang",
    "zig-package",
    "std",
    "stdlib",
    "standard library",
    "comptime",
    "build.zig",
    "zigmod",
    "install",
    "installation",
    "setup",
    "configure",
    "configuration",
    "config",
    "usage",
    "howto",
    "guide",
    "tutorial",
    "walkthrough",
    "docs",
    "documentation",
    "readme",
    "changelog",
    "release",
    "releases",
    "version",
    "v1",
    "v2",
    "v3",
    "dependency",
    "dependencies",
    "dep",
    "deps",
    "registry",
    "index",
    "manifest",
    "lockfile",
    "github",
    "gitlab",
    "codeberg",
    "bitbucket",
    "git",
    "commit",
    "commits",
    "branch",
    "fork",
    "pull request",
    "pr",
    "issue",
    "issues",
    "tags",
    "tag",
    "algorithm",
    "data structure",
    "utils",
    "helper",
    "helpers",
    "core",
    "base",
    "main",
    "app",
    "application",
    "service",
    "server",
    "client",
    "backend",
    "frontend",
    "api",
    "sdk",
    "the",
    "a",
    "an",
    "and",
    "or",
    "of",
    "for",
    "in",
    "on",
    "with",
    "by",
    "to",
    "from",
    "using",
    "written",
    "cross platform",
    "cross-platform",
    "portable",
    "high performance",
    "fast",
    "lightweight",
    "minimal",
    "simple",
    "easy",
    "best",
    "awesome",
    "collection",
    "list",
    "curated",
]


kw_model = None
stop_words = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global kw_model, stop_words
    nltk.download("stopwords")
    kw_model = KeyBERT()

    stop_words = stopwords.words("english")
    stop_words.extend(ZIG_STOPWORDS)
    yield


from pydantic import BaseModel

app = FastAPI(lifespan=lifespan)


class ReadmeContent(BaseModel):
    text: str


@app.get("/extract_keywords")
async def index(payload: ReadmeContent):
    readme_content = payload.text.trim()
    number_of_words = len(readme_content.split())

    if number_of_words == 0:
        return {"keywords": ""}

    # Only 15% i.e, if readme has 100 words, 15
    # most important words would be selected.
    number_of_words_to_choose = int(number_of_words * (15 / 100))

    keywords = kw_model.extract_words(
        readme_content,
        keyphrase_ngram_range=(1, 1),
        stop_words=stop_words,
        top_n=number_of_words_to_choose,
    )

    all_list_of_output = [i[0] for i in keywords[:200]]

    processed_keywords = " ".join(all_list_of_output)

    return {"keywords": processed_keywords}
