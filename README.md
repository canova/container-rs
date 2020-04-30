# Container-rs

This is a work in progress implementation of Linux container runtime in Rust. Currently only supports Linux. You need a VM to be able to run under macOS or Windows.

## Run

You can run a new container with the following command:

```bash
sudo ./run.sh run bash
```

Currently we don't support different file systems. So you need to create a new file system to your project home directory with the directory name called `new_ubuntu`. That will be changed soon.

You can download a simple ubuntu file system from the docker registry or you can just download [this tar.gz][tarfile] file directly. This is the file system the ubuntu docker image is based on.

[tarfile]: https://github.com/tianon/docker-brew-ubuntu-core/blob/fbcb3103ee22258b052bd7978989a302230ac5e5/bionic/ubuntu-bionic-core-cloudimg-amd64-root.tar.gz
