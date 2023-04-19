# SharePrompts

Extension to share your ChatGPT conversations and prompts.

Inspired by [ShareGPT](https://github.com/domeccleston/sharegpt), does not share any code with that project.

## Tech

Backend is Rust with Postgres db. Frontend is JavaScript. Learning as I go here.


## Database backup/restore

Command to make a snapshot db backup (saved locally in timestamped file):

    ssh root@shareconversation.com "su - postgres -c \"pg_dump --clean sharedprompts\"" > sharedprompts-`date +"%FT%H%M"`.sql

To restore from backup, assuming backup file is locally copied to `/var/lib/postgresql/` location:

    sudo su - postgres
    psql -f sharedprompts-XXX.sql

With the `--clean` in the backup, this will wipe the DB and go directly to backup state.

Also note that restoring from backups requires stopping the backend API server. Can't drop the database until that is done.