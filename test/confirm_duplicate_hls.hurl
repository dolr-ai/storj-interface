# Verify that HLS files were uploaded correctly
# This script checks that the HLS files exist in the appropriate buckets

# Confirm master.m3u8 exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/master.m3u8"
}
HTTP 200

# Confirm playlist_0.m3u8 exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/playlist_0.m3u8"
}
HTTP 200

# Confirm playlist_1.m3u8 exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/playlist_1.m3u8"
}
HTTP 200

# Confirm playlist_2.m3u8 exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/playlist_2.m3u8"
}
HTTP 200

# Confirm data_0_000.ts exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/data_0_000.ts"
}
HTTP 200

# Confirm data_1_000.ts exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/data_1_000.ts"
}
HTTP 200

# Confirm data_2_000.ts exists in SFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-videos/{{video_id}}/hls/data_2_000.ts"
}
HTTP 200

# Confirm master.m3u8 exists in NSFW bucket
GET {{host}}/confirm_duplicate
Authorization: Bearer {{api_token}}
{
  "uri": "sj://yral-nsfw-videos/{{video_id}}_nsfw/hls/master.m3u8"
}
HTTP 200