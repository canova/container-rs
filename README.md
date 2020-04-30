# Container-rs

This is a work in progress implementation of Linux container runtime in Rust. Currently only supports Linux. You need a VM to be able to run under macOS or Windows.

## Run

You can run a new container with the following command:

```bash
./run.sh run resources/ubuntu-fs.tar.gz bash
```

This builds and runs the container runtime but you need to be a sudo user and put your password to be able to run it. Because during the process creation, we need a privileged user.

We support different file systems. To test the project, you can use the ubuntu file system inside the resources directory. You can also specify your own file system like this:

```bash
./run.sh run <path-to-your-fs> bash
```

You can also download a simple ubuntu file system from the docker registry or you can just download [this tar.gz][tarfile] file directly. This is the file system the ubuntu docker image is based on. Docker registry support will come soon.

[tarfile]: https://github.com/tianon/docker-brew-ubuntu-core/blob/fbcb3103ee22258b052bd7978989a302230ac5e5/bionic/ubuntu-bionic-core-cloudimg-amd64-root.tar.gz
