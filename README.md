# unishort
An URL shortener that uses (almost) the full Unicode range to create the shortest URLs possible.

The URLs created by this shortener are only seven characters long, about half as long as the URLs of bit.ly

Hosted at [🤏.to](http://🤏.to)

## Todo
- [x] Proof of concept
- [x] Accept scheme-less URLs (google.com instead of https://google.com)
- [ ] Improve design
- [ ] Write an API specification (should be very easy)

## Deployment

Runs on the `sun` k3s cluster. Pushing to `master` triggers
`.github/workflows/build-deploy.yml`, which builds the image, pushes it to the
self-hosted Zot registry, and commits the new image tag to
[sun-gitops](https://github.com/vongaisberg/sun-gitops); Argo CD then rolls it
out. Tagging a commit `v*` additionally publishes that semver tag.

Reachable at `app-unishort.vongaisberg.de` publicly and
`unishort.dachshund-minor.ts.net` over Tailscale.

## Branches

`master` deploys to production at `https://app-unishort.vongaisberg.de`.

`dev` deploys to `https://app-unishort-dev.vongaisberg.de`, which sits behind a
Keycloak login — external contributors need an account in the `sun` realm's
`apps-users` group. Both branches build through the same pipeline and push
SHA-tagged images to the same registry; only the manifest they update differs.
