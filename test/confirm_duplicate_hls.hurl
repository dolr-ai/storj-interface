# Verify that HLS files were uploaded correctly
# SFW HLS files now go to Hetzner S3, so we can't check via Storj share URL
# Only verify NSFW HLS files which still use Storj

# Confirm master.m3u8 exists in NSFW bucket
HEAD {{nsfw_share}}/{{video_id}}_nsfw/hls/master.m3u8
HTTP 200