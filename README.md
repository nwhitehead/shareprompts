# SharePrompts

Extension to share your ChatGPT conversations and prompts.

Inspired by [ShareGPT](https://github.com/domeccleston/sharegpt), does not share
any code with that project.

## Tech

Backend is Rust with Postgres db. Frontend is JavaScript. Learning as I go here.


## Database backup/restore

Command to make a snapshot db backup (saved locally in timestamped file):

    ssh root@shareconversation.com "su - postgres -c \"pg_dump --clean sharedprompts\"" > sharedprompts-`date +"%FT%H%M"`.sql

To restore from backup, assuming backup file is locally copied to `~postgres`
location:

    sudo su - postgres -c "psql -d sharedprompts -f sharedprompts-XXX.sql"

WARNING: With the `--clean` in the backup generation, this will wipe the DB and
go directly to backup state. Also note that restoring from backups requires
stopping the backend API server. Can't drop the database until that is done.

## Authentication

The Chrome extension is an "app" in the Chrome developer console so has its own
application id and everything. From within the extension we can access
`chrome.identity` and request authentication tokens. The scopes are controlled
by the manifest file. These tokens are full "access tokens". I don't see a way
to just get id tokens.

To validate access tokens, I use:

    https://www.googleapis.com/oauth2/v1/tokeninfo?access_token=...


For the website, there are some Google Identity Services for Web components.
Basically you load the Google Client API JavaScript and set up a `div` with some
fields and it will make it a nice "Sign in with Google" button. The result of
this flow is an id token.

To validate the id token you need the Google public keys. Google keys are in
JWKS (JSON Web Key Set) format at:

    https://www.googleapis.com/oauth2/v3/certs

The server caches them and uses cached values. The server grabs the keys every
few hours whatever the cache control headers say.

## Zip up extension

Need to do a build step:

    cd extension
    npm run build
    cd ..
    zip -r extension.zip extension/manifest.json extension/dist/* extension/images/*


## Currently working on

* User conversation counts, for free/paid check
* Integrating with extpay
* Updating previously shared conversations if you share same convo again
