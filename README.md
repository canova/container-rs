# Container-rs

This is a work in progress implementation of Linux container runtime in Rust. Currently only supports Linux. You need a VM to be able to run under macOS or Windows.

## Run

You can download a Docker image and run a new container with the following commands:

```bash
./run.sh pull library/ubuntu # Get the ubuntu image from the docker registry.
./run.sh run library/ubuntu bash # Create a new container and run bash with the ubuntu image.
```

or you can also specify your own file system tarball like this:

```bash
./run.sh run resources/ubuntu-fs.tar.gz bash
```

This builds and runs the container runtime but you need to be a sudo user and put your password to be able to run it. Because during the process creation, we need a privileged user.

You can list the downloaded images with:

```bash
./run.sh images
```

and remove an image with:

```bash
./run.sh images -r library/ubuntu
```
