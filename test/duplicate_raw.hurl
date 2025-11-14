# ============================================================================
# Step 1: Initial Upload (with TTL)
# ============================================================================

# Initial upload - SFW video with 1h TTL
POST {{host}}/duplicate_raw/upload?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200
[Asserts]
jsonpath "$.status" == "pending"
jsonpath "$.expires_in_hours" == 1

# Initial upload - NSFW video
POST {{host}}/duplicate_raw/upload?publisher_user_id={{publisher}}&video_id={{video_id}}_raw_nsfw&is_nsfw=true
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200
[Asserts]
jsonpath "$.status" == "pending"

# Initial upload - Without auth token
POST {{host}}/duplicate_raw/upload?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 401

# Initial upload - With wrong token
POST {{host}}/duplicate_raw/upload?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer wrongtoken
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 401

# Initial upload - missing required query parameter publisher_user_id
POST {{host}}/duplicate_raw/upload?video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 400

# Initial upload - missing required query parameter video_id
POST {{host}}/duplicate_raw/upload?publisher_user_id={{publisher}}&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 400

# Initial upload - missing required query parameter is_nsfw
POST {{host}}/duplicate_raw/upload?publisher_user_id={{publisher}}&video_id={{video_id}}_raw
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 400

# ============================================================================
# Step 2: Finalize Upload (with metadata, removes TTL)
# ============================================================================

# Finalize - SFW video with metadata
POST {{host}}/duplicate_raw/finalize?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer {{api_token}}
{
  "metadata": {
    "test": "value",
    "title": "Test Video"
  }
}
HTTP 200
[Asserts]
jsonpath "$.status" == "completed"

# Finalize - NSFW video with metadata
POST {{host}}/duplicate_raw/finalize?publisher_user_id={{publisher}}&video_id={{video_id}}_raw_nsfw&is_nsfw=true
Authorization: Bearer {{api_token}}
{
  "metadata": {
    "test": "value"
  }
}
HTTP 200
[Asserts]
jsonpath "$.status" == "completed"

# Finalize - Without auth token
POST {{host}}/duplicate_raw/finalize?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
{
  "metadata": {
    "test": "value"
  }
}
HTTP 401

# Finalize - With wrong token
POST {{host}}/duplicate_raw/finalize?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer wrongtoken
{
  "metadata": {
    "test": "value"
  }
}
HTTP 401
