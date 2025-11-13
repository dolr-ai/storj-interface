# SFW videos now go to both Storj and Hetzner S3
# Check SFW video exists in Storj (from /duplicate endpoint)
HEAD {{sfw_share}}/{{publisher}}/{{video_id}}.mp4
HTTP 200

# Check NSFW video exists in Storj (from /duplicate endpoint)
HEAD {{nsfw_share}}/{{publisher}}/{{video_id}}.mp4
HTTP 200

# Check SFW video exists in Storj (from /duplicate_raw endpoint with metadata)
HEAD {{sfw_share}}/{{publisher}}/{{video_id}}_raw.mp4
HTTP 200

# Check SFW video exists in Storj (from /duplicate_raw endpoint without metadata)
HEAD {{sfw_share}}/{{publisher}}/{{video_id}}_raw_no_meta.mp4
HTTP 200

# Check NSFW video exists in Storj (from /duplicate_raw endpoint)
HEAD {{nsfw_share}}/{{publisher}}/{{video_id}}_raw_nsfw.mp4
HTTP 200