# SFW videos now go to both Storj and Hetzner S3
# Check SFW video exists in Storj
HEAD {{sfw_share}}/{{publisher}}/{{video_id}}_raw.mp4
HTTP 200

# Check NSFW video exists in Storj
HEAD {{nsfw_share}}/{{publisher}}/{{video_id}}_raw_nsfw.mp4
HTTP 200
