use zigistry::keyword_extraction;

#[tokio::main]
async fn main() {
    let readme_content = r#"

# Zigistry

**A place where you can find all the libraries and programs that suit your Zig
lang needs.**

---

## Adding your repo to Zigistry:

Either on GitHub or on Codeberg:

Go to your Repository:

Add `zig-package` topic to it, if it is a Zig library. Or Add `zig`
topic to it, if it is a Zig application/program.

> [!IMPORTANT]
> Then create any commit and push.

## Contribution:

- Feel free to create a Pull request, mention an issue or suggest any features
  that you want Zigistry should have or improve.

## Code of conduct:

- Zigistry follows the
  [Zig Code of Conduct](https://ziglang.org/code-of-conduct/)        
    "#;
    let description = r#"

 A place where you can find all the libraries that suit your Zig lang needs. ⭐️ Please star to support this work!
   
 "#;

    let repo_name = "Zigistry";
    let owner_name = "Zigistry";
    let res = keyword_extraction(readme_content, description, repo_name, owner_name)
        .await
        .unwrap();

    println!("{}", res);
}
