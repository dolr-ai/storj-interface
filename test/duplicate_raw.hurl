# proper input - SFW video with metadata
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false&metadata=%7B%22test%22%3A%22value%22%7D
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200

# proper input - SFW video without metadata
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw_no_meta&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200

# Segregate nsfw videos
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw_nsfw&is_nsfw=true
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200

# Without auth token
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 401

# With wrong token
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer wrongtoken
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 401

# missing required query parameter - publisher_user_id
POST {{host}}/duplicate_raw?video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 400

# missing required query parameter - video_id
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 400

# missing required query parameter - is_nsfw
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 400
