## Shuttle SNS Example
This repository shows how you can deploy a Shuttle example that uses AWS SNS.

## Deployment
To deploy, clone this repo then go to the folder of the project you want to deploy.

You'll need to set the project name by setting it in `Shuttle.toml` in the repo:

``` toml
name = "my_project_name_example"
```

Then make sure you're setting the secrets in Secrets.toml as per the article.

Once done, use `cargo shuttle project start` and `cargo shuttle deploy --allow-dirty` to deploy!
