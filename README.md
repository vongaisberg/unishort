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
