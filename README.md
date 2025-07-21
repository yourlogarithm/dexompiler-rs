### dexompiler

This is a Rust library developed as a part of my bachelor's degree final project.

It exposes a structure called `Apk` and a function `parse` that takes as input an `.apk` buffer and returns an `Apk` object containing:
* Package name
* Permissions
* Components
  * Activities
  * Services
  * Receivers
  * Providers
* A vector of `Method` each containing the method signature and a vector of opcodes used by method. The method is sorted using [Depth-First search](https://en.wikipedia.org/wiki/Depth-first_search) prioritizing manifest components' methods first.

#### Example

```rust
use dexompiler::parse;
use std::fs::File;

fn main() {
    let file = File::open("F-Droid.apk").unwrap();
    let apk = parse(file).unwrap();
    if let Some(manifest) = apk.manifest.as_ref() {
        println("Package name: {:?}", manifest.package);
        println("Permissions: {:?}", manifest.permissions);
        println("Activities: {:?}", manifest.activities);
        println("Services: {:?}", manifest.services);
        println("Receivers: {:?}", manifest.receivers);
        println("Providers: {:?}", manifest.providers);
    }
    for method in apk.methods {
        println!("{} -> {} instructions", method.fullname, method.insns.len());
    }
}
```
