# Upload master.m3u8 file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "master.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/master.m3u8;
HTTP 200

# Upload playlist_0.m3u8 file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "playlist_0.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/playlist_0.m3u8;
HTTP 200

# Upload playlist_1.m3u8 file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "playlist_1.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/playlist_1.m3u8;
HTTP 200

# Upload playlist_2.m3u8 file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "playlist_2.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/playlist_2.m3u8;
HTTP 200

# Upload data_0_000.ts file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "data_0_000.ts", "metadata": {"test": "value"}}
file: file,test-hls-files/data_0_000.ts;
HTTP 200

# Upload data_1_000.ts file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "data_1_000.ts", "metadata": {"test": "value"}}
file: file,test-hls-files/data_1_000.ts;
HTTP 200

# Upload data_2_000.ts file
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "data_2_000.ts", "metadata": {"test": "value"}}
file: file,test-hls-files/data_2_000.ts;
HTTP 200

# Test with NSFW content
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}_nsfw", "is_nsfw": true, "hls_file_name": "master.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/master.m3u8;
HTTP 200

# Test without auth token
POST {{host}}/duplicate_hls
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "master.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/master.m3u8;
HTTP 401

# Test with wrong token
POST {{host}}/duplicate_hls
Authorization: Bearer wrongtoken
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "master.m3u8", "metadata": {"test": "value"}}
file: file,test-hls-files/master.m3u8;
HTTP 401

# Test missing data field
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
file: file,test-hls-files/master.m3u8;
HTTP 400

# Test missing file field
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"publisher_user_id": "{{publisher}}", "video_id": "{{video_id}}", "is_nsfw": false, "hls_file_name": "master.m3u8", "metadata": {"test": "value"}}
HTTP 400

# Test with invalid JSON in data field
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: malformed json
file: file,test-hls-files/master.m3u8;
HTTP 400

# Test with missing required fields in JSON
POST {{host}}/duplicate_hls
Authorization: Bearer {{api_token}}
[Multipart]
data: {"video_id": "{{video_id}}", "is_nsfw": false}
file: file,test-hls-files/master.m3u8;
HTTP 400