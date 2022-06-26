set dotenv-load	

spin-up:
  spin build && spin up --follow-all -e DISCORD_PUB_KEY=$DISCORD_PUB_KEY -e DISCORD_BOT_TOKEN=$DISCORD_BOT_TOKEN

