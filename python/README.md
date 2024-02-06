# Setup

The build process relies on [Hatchling](https://hatch.pypa.io/latest/) for
building, packaging, and deployment. If you want to perform type checking or
ensure lint-freeness, you'll also need to install
[`mypy`](https://github.com/python/mypy) and
[`pylint`](https://github.com/pylint-dev/pylint). Installation of Python tools
can follow many different paths, depending on whether global, local, or virtual
toolchains are preferred, so precise installation details are beyond the scope
of this document.

You will also need to install the [AWS CLI](https://aws.amazon.com/cli/) and
configure it appropriately for your AWS environment. This, too, is beyond the
scope of this simply document.

# Deployment

Once you have installed all tools mentioned above, you can use Hatchling to
package and deploy the services.

## `events-a`

Change the current working directory to `events-a`.

You can then package and deploy thus:

```shell
hatch run package
```

You can use the custom `create` script to create and publish the remote AWS
Lambda, but you will likely need to modify the `--role` parameter. Once
customized for your AWS environment, you can:

```shell
hatch run create
```

If you need to redeploy (because you made changes):

```shell
hatch run package
hatch run deploy
```

## `events-b`

Change the current working directory to `events-b`.

You can then package and deploy thus:

```shell
hatch run package
```

You can use the custom `create` script to create and publish the remote AWS
Lambda, but you will likely need to modify the `--role` parameter. Once
customized for your AWS environment, you can:

```shell
hatch run create
```

If you need to redeploy (because you made changes):

```shell
hatch run package
hatch run deploy
```
