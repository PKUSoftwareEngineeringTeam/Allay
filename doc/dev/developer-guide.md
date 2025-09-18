# Developer Documentation

## About Allay Engine

Want to build a beautiful personal blog but don't want to write a lot of HTML and CSS? Allay Engine is
suit for you. With Allay Engine, all you need is markdown! Allay Engine can automatically translate
your markdown into HTML based on the theme you choose. Also, Allay Engine is flexible and configurable. You and write your own theme and shortcode.

## Get Started

### Step 1: Install Rust Toolchain

Please follow the [official guide](https://www.rust-lang.org/tools/install) provided by Rust developer team.
You may need to install some other tools, for example, Microsoft Visual Studio on Windows for the linker.

> **Note**: Make sure your Rust toolchain is always *latest stable* version. Other versions of Rust
> may work, but our CI/CD uses the latest stable version, so some checks may fail.
>
> For typical installation of Rust, you can run `rustup update` to update.
> 
> If you are a beginner of Rust, see the guide in [Rust book](https://doc.rust-lang.org/book/). You
> can also read the Chinese translation at [here](https://kaisery.github.io/trpl-zh-cn/), but it may be outdated.

### Step 2: Be Familiar with the Code Base

After installing Rust, you can start with the code base. It's highly recommended to read the documentations in
`doc` directory first, but you can also start with crates in `crates` directory. For more detailed
information about the project structure, please read the Project Structure section.

### Step 3: Implement Features or Fix Bugs

Since directly push to main branch is not allowed, you need to create a new branch. You can also
create an issue on GitHub if your work is complex. Please make sure your commit message contains
valuable information. Commit messages like "fix", "update" or "test" should not appear. If your commit
message is not valuable, there is a high probability that your pull request will be rejected.

If you are working on a new feature, please add tests if possible. If you are fixing a bug, you should
add regression tests. Documents matters too, it's highly recommended to add Rust doc to structs, functions
or traits exported.

> **About Unsafe Code**: I believe there are many times Rust's ownership system and borrow checker
> complain your code, and you want to write code freely just like C/C++. Although `unsafe` allows
> you to do that, but think about it first! Why the Rust compiler complains your code? Is your code
> really unsafe and lead to undefined behaver? Many times the Rust compiler is right. If you still
> think you are correct, use `unsafe` code carefully and add a comment about it's safety to it.
> Keep in mind that `unsafe` means "I'm sure it is safe".

### Step 4: Pull Request and Code Review

You can open a pull request once your work is finished. When you open a pull request, GitHub will
run checks automatically. Pull request with failed CI/CD is not allowed, you need to change your code.
If you are sure who will be the best person to review the code, make them a reviewer. Otherwise, leave
the reviewer blank.

> **Note**: Currently, our CI/CD contains checking for code format, code style and unit tests. If
> some check fails when you open a pull request, don't worry, you can fix it!
> 
> - If task `fmt` failed, it means you code style is wrong, run `cargo fmt` to fix it.
> - If task `Clippy` failed, it means your code contains some performance, readability or maintainability
>   issues, run `cargo clippy` to check them. For most lint errors, cargo can fix them automatically,
>   use `cargo clippy --fix`
> - If task `Test` failed, it means your unit tests failed. You may either break exist code base or
>   there is some bug in your new code. You can run `cargo test` to see what's wrong and fix them.
> 
> After you fix your code, you can simply push your code to the same branch. This will add your fix
> to the pull request automatically.

For reviewers, you need to read the pull request and check whether there is some potential vulnerability
of the code. Then you should make your choice: *requested changes* or *approved*. For non-requested
reviewers, you can also leave a comment to help the reviewers. When the code is ready for merge, make
sure you use the "Squash and merge" provided by GitHub.

## Project Structure

This project is managed by Cargo Workspace, which means there are more than one crate in this project.
Here is an overview about all crates and their function.

- `allay-base`: Basic utilities for the blog engine, such as user configuration, data structure and
  IO utilities.
- `allay-compiler`: Compiler which turn markdown and HTML template into final HTML code.
  See [compiler.md](compiler.md) for more information.
- `allay-cli`: Command line interface for users. It provides commends such as `allay init` and `allay new`.
- `allay-publish`: Publish compile result to artifact folder. It also listens file changes to support
  hot-reload.
- `allay-backend`: An HTTP server for live preview.