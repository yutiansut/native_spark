# Introduction

`native_spark` is a distributed computing framework inspired by Apache Spark.

## Getting started

### Installation

Right now the framework lacks any sort of cluster manager of submit program/script.

In order to use the framework you have to clone the repository and add the local dependency or add the upstream GitHub repository to your Rust project (the crate is not yet published on [crates.io](https://crates.io/)). E.g. add to your application Cargo.toml or:

```doc
[dependencies]
native_spark = { path = "/path/to/local/git/repo" }
# or
native_spark = { git = "https://github.com/rajasekarv/native_spark", branch = "master }
```

Is not recommended to use the application for any sort of production code yet as it's under heavy development.

Check [examples](https://github.com/rajasekarv/native_spark/tree/master/examples) and [tests](https://github.com/rajasekarv/native_spark/tree/master/tests) in the source code to get a basic idea of how the framework works.

## Executing an application

In order to execute application code some preliminary setup is required. (So far only tested on Linux.)

* Install [Cap'n Proto](https://capnproto.org/install.html). Required for serialization/deserialziation and IPC between executors.
* If you want to execute examples, tests or contribute to development, clone the repository `git clone https://github.com/rajasekarv/native_spark/`, if you want to use the library in your own application you can just add the depency as indicated in the installation paragraph.
* You need to have [hosts.conf](https://github.com/rajasekarv/native_spark/blob/master/config_files/hosts.conf) in the format present inside config folder in the home directory of the user deploying executors in any of the machines.
    * In `local` mode this means in your current user home, e.g.:
    > $ cp native_spark/config_files/hosts.conf $HOME
    * In `distributed` mode the same file is required in each host that may be deploying executors (the ones indicated in the `hosts.conf` file) and the master. E.g.:
    ```doc
    $ ssh remote_user@172.0.0.10 # this machine IP is in hosts.conf
    # create the same hosts.conf file in every machine:
    $ cd ~ && vim hosts.conf ...
    ```
* The environment variable `NS_LOCAL_IP` must be set for the user executing application code.
    * In `local` it suffices to set up for the current user:
    > $ export NS_LOCAL_IP=0.0.0.0
    * In `distributed` the variable is required, aditionally, to be set up for the users remotely connecting. Depending on the O.S. and ssh defaults this may require some additional configuration. E.g.:
    ```doc
    $ ssh remote_user@172.0.0.10
    $ sudo echo "NS_LOCAL_IP=172.0.0.10" >> .ssh/environment
    $ sudo echo "PermitUserEnvironment yes" >> /etc/ssh/sshd_config
    $ service ssh restart 
    ```

Now you are ready to execute your application code; if you want to try the provided 
examples just run them. In `local`:
> cargo run --example make_rdd

In `distributed`:
> cargo run --example make_rdd -d distributed

## Deploying with Docker

There is a docker image and docker-compose script in order to ease up trying testing 
and deploying distributed mode on your local host. In order to use them:

1. Build the examples image under the repository `docker` directory:
> bash docker/build_image.sh

2. When done, you can deploy a testing cluster:
> bash testing_cluster.sh

This will execute all the necessary steeps to to deploy a working network of containers where you can execute the tests. When finished you can attach a shell to the master and run the examples:
```doc
$ docker exec -it docker_ns_master_1 bash
$ ./make_rdd -d distributed
```

## Setting execution mode

In your application you can set the execution mode (`local` or `distributed`) in one of the following ways:

1. Set it explicitly while creating the context, e.g.:
```doc
    use native_spark::DeploymentMode;

    let context = Context::with_mode(DeploymentMode::Local)?;
```
2. Execute the application with the `deployment mode` argument set to one of the valid modes (e.g.: `./my_app -d distributed`)
3. Set the DEPLOYMENT_MODE environment variable (e.g.: `DEPLOYMENT_MODE=local`

### Additional notes

Since File readers are not done, you have to use manual file reading for now (like manually reading from S3 or hack around local files by distributing copies of all files to all machines and make rdd using filename list).

Ctrl-C and panic handling are not done yet, so if there is some problem during runtime, executors won't shut down automatically and you will have to manually kill the processes.

One of the limitations of current implementation is that the input and return types of all closures and all input to make_rdd should be owned data.
