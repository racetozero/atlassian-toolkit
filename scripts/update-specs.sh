#!/usr/bin/env bash
# Refresh the vendored Atlassian OpenAPI documents used to generate the
# command catalog. Run from the repository root, then `cargo test -p atl-sdk`.
set -euo pipefail

dest="crates/atl-sdk/specs"
cloud="https://dac-static.atlassian.com/cloud"
server="https://dac-static.atlassian.com/server"

while read -r file url; do
  echo "fetch $file"
  curl --fail --silent --show-error --location --output "$dest/$file.tmp" "$url"
  # Atlassian's sample access tokens look real to secret scanners (GitHub
  # push protection rejects them), so the examples become a placeholder.
  perl -pi -e 's/("token":\{"type":"string","example":")[^"]+/$1<token>/g' "$dest/$file.tmp"
  mv "$dest/$file.tmp" "$dest/$file"
done <<LIST
jira-cloud.json $cloud/jira/platform/swagger-v3.v3.json
jira-software-cloud.json $cloud/jira/software/swagger.v3.json
jsm-cloud.json $cloud/jira/service-desk/swagger.v3.json
confluence-v2-cloud.json $cloud/confluence/openapi-v2.v3.json
confluence-v1-cloud.json $cloud/confluence/swagger.v3.json
bitbucket-cloud.json $cloud/bitbucket/swagger.v3.json
trello-cloud.json $cloud/trello/swagger.v3.json
admin-organization-cloud.json $cloud/admin/organization/swagger.v3.json
admin-user-management-cloud.json $cloud/admin/user-management/swagger.v3.json
admin-api-access-cloud.json $cloud/admin/api-access/swagger.v3.json
admin-control-cloud.json $cloud/admin/control/swagger.v3.json
admin-dlp-cloud.json $cloud/admin/dlp/swagger.v3.json
admin-user-provisioning-cloud.json $cloud/admin/user-provisioning/swagger.v3.json
jira-dc.json $server/jira/platform/jira_software_dc_11003_swagger.v3.json
confluence-dc.json $server/confluence/10.2.20.swagger.v3.json
bitbucket-dc.json $server/bitbucket/10.5.swagger.v3.json
LIST
