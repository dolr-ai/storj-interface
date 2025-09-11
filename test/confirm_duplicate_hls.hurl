# Verify that HLS files were uploaded correctly
# SFW HLS files now go to both Storj and Hetzner S3

# Check SFW HLS master.m3u8 exists in Storj
HEAD {{sfw_share}}/{{video_id}}/hls/master.m3u8
HTTP 200

# Check NSFW HLS master.m3u8 exists in Storj
HEAD {{nsfw_share}}/{{video_id}}_nsfw/hls/master.m3u8
HTTP 200