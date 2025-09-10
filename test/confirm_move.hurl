# SFW videos are now in S3, can't verify removal via Storj share URL
# Only verify that video exists in NSFW bucket after move
HEAD {{nsfw_share}}/{{publisher}}/{{video_id}}.mp4
HTTP 200
