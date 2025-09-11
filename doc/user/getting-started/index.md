# Create Your First Blog

Welcome to the Allay Blog Engine! This guide will walk you through the steps to create your first blog using Allay.

## Step 1: Install Allay

To get started, you'll need to install Allay first. You can refer to [github release page](https://github.com/PKUSoftwareEngineeringTeam/Allay/releases) for pre-built binaries, or you may build the program by yourself. If so, clone the repository:

```sh
git clone https://github.com/PKUSoftwareEngineeringTeam/Allay
```

Build the project using Cargo:

```sh
cd Allay
cargo build --release
```

This will create an executable in the `target/release` directory.

Check the executable version to ensure it's installed correctly:

```sh
./target/release/allay --version
```

You may add the executable to your system PATH for easier access.

## Step 2: Get a Demo

Our demo blog is available on [GitHub](https://github.com/PKUSoftwareEngineeringTeam/Axolotl-Wiki). Clone it to see a working example of an Allay-powered blog.

```sh
git clone https://github.com/PKUSoftwareEngineeringTeam/Axolotl-Wiki
cd Axolotl-Wiki
```

## Step 3: Serve the Blog

To serve the blog locally, navigate to the blog directory and run the following command:

```sh
allay server
```

This will generate the pages in `public` directory and start a local server. You can access your blog at `http://localhost:8000` by default. The port may be changed if it is already in use.

Change the content in the `content` directory, and enjoy your blog!
