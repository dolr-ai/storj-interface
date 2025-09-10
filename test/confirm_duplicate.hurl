# SFW videos now go to S3, so we can't check via Storj share URL
# Only verify NSFW videos which still use Storj
HEAD {{nsfw_share}}/{{publisher}}/{{video_id}}.mp4
HTTP 200
