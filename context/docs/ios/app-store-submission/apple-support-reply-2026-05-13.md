Apple Support Reply — 2026-05-13

STATUS: SENT 2026-05-13. Apple replied 2026-05-22 recommending upgrade to Xcode 26.5 (released 2026-05-11). After upgrading, build 13 cleared "Add for Review" and is now in "Waiting for Review". Case resolved. See appstore-debug.md for full resolution. Now we wait!

Reply to case 102856406657 (Xue) following the request to upload a new build with a higher build number and provide a video. Includes API-level evidence captured from browser DevTools.


Subject (keep the existing thread):
Re: Case 102856406657 — Build 12 uploaded, "Add for Review" still rejected (API-level evidence attached)


Hi Xue,

Thanks for the response. I followed your instructions and uploaded a new build with a higher build number. Here is the new evidence, which I believe will help engineering pinpoint the root cause.


1. The build is not stuck processing.

Per your request, I uploaded build 12 today.

- Prerelease version / build: 1.0 (12)
- Uploaded: May 13, 2026, 4:42 PM PDT via Transporter (TransporterApp/1.4-14025)
- Delivery UUID: 066233d2-7e25-41d6-9282-b9f40fa53bc3
- Upload result: zero errors, zero warnings

TestFlight shows build 12 as Complete, alongside builds 8 through 11 from earlier attempts. All five recent builds reached the "Complete" state in TestFlight within minutes of upload.

[IMAGE 1: TestFlight screen — Zwipe MTG, TestFlight tab, showing builds 12, 11, 10, 9, 8 all marked "Complete"]

A short screen recording walking through this flow is also attached for reference.

[VIDEO: screen recording — full flow from TestFlight "Complete" to Distribution "Add for Review" to rejection]


2. The "Add for Review" rejection in the UI.

When I click Add for Review in the Distribution tab, App Store Connect displays the following message:

"Unable to Add for Review. New apps and app updates must be built with the latest public (GM) versions of Xcode, and the iOS, macOS, watchOS, and tvOS SDKs. Apps built with beta versions aren't allowed."

[IMAGE 2: Distribution tab — Zwipe MTG, iOS App Version 1.0, showing the red "Unable to Add for Review" banner]

The same backend rejection also surfaces in a second UI path: clicking "Add Draft" produces a "Draft Submissions (1)" toast that auto-deletes within about 2 seconds. The Draft Submission dialog briefly shows "No Items" and "Delete Submission" before closing.

[IMAGE 3: Draft Submission dialog — showing "No Items" and "Delete Submission" right before it auto-closes]


3. The actual API-level error tells a different story.

I inspected the failing network request in browser DevTools. The user-facing "beta Xcode" message is a translation of an entirely different backend error code:

- Endpoint: POST https://appstoreconnect.apple.com/iris/v1/reviewSubmissionItems
- HTTP status: 409
- Error code: ENTITY_ERROR.RELATIONSHIP.INVALID.INVALID_STATE.BUILD_SDK_NOT_ALLOWED_FOR_APP_STORE_SUBMISSION
- Title: "The build's SDK build is not supported yet."
- Detail: "Build SDK build is not yet supported."
- Timestamp: Wed, 13 May 2026 23:50:41 GMT

[IMAGE 4: DevTools Network panel — Response tab, showing the JSON 409 error body with BUILD_SDK_NOT_ALLOWED_FOR_APP_STORE_SUBMISSION]

Apple trace headers from the failed response (for engineering to look up the exact server-side check that fired):

- x-apple-jingle-correlation-key: 2F4SJIS3WXIFJ4UVKHOU567OEQ
- x-apple-request-uuid: d17924a2-5bb5-d054-f295-51dd4efbee24
- b3 traceid: d17924a25bb5d054f29551dd4efbee24-144ce7593c2a703a
- apple-originating-system: UnknownOriginatingSystem
- apple-timing-app: 1296 ms

[IMAGE 5: DevTools Network panel — Headers tab, showing the response headers including the correlation key]

Affected resources:

- App Store Version ID: 2a509063-12a7-4f6f-a756-20e4ca381bf2
- Review Submission ID: 57161d5e-31d1-4cea-bf04-945547756536

The error code BUILD_SDK_NOT_ALLOWED_FOR_APP_STORE_SUBMISSION and the phrase "not yet supported" suggest a server-side SDK allowlist is rejecting the build, rather than any actual beta-toolchain detection.


4. Context already shared with Liping (case 102855955579).

- 12 builds uploaded across 2 bundle IDs (com.scadoshi.zwipe, com.scadoshi.zwipetest) and 2 Xcode GM versions (26.3 build 17C529 and 26.4 build 17E192, both installed from the Mac App Store)
- A native Swift binary compiled with xcrun -sdk iphoneos swiftc (no Rust, no Dioxus, no third-party frameworks of any kind) receives the same rejection, so this is not toolchain-specific
- xcrun altool --validate-app passes with zero errors on every build
- Uploads have been attempted via both altool and Transporter, same rejection in both cases
- Liping previously offered to escalate to engineering once the upload-method suggestion had been tried


Requests:

1. Please merge cases 102856406657 and 102855955579 so we are not duplicating effort across two engineers.
2. Please escalate to engineering with the correlation key 2F4SJIS3WXIFJ4UVKHOU567OEQ so they can trace the exact allowlist check at iris/v1/reviewSubmissionItems that is rejecting App Store Version 2a509063-12a7-4f6f-a756-20e4ca381bf2.

App ID: 6761341603, Team ID: VV74WQ89GD.

Thanks,
Scotty


Attachment checklist (notes for Scotty, not part of the email):

1. IMAGE 1 — TestFlight screen, build 12 plus history all "Complete". Source: App Store Connect, TestFlight tab.
2. IMAGE 2 — Distribution rejection, red "Unable to Add for Review" banner. Source: App Store Connect, Distribution, iOS App Version 1.0.
3. IMAGE 3 — Draft auto-delete, "No Items" / "Delete Submission" dialog. Source: Distribution, Add Draft, toast appears about 2 seconds then closes.
4. IMAGE 4 — DevTools Response, 409 JSON body. Source: Firefox/Chrome DevTools, Network, the failing POST, Response tab.
5. IMAGE 5 — DevTools Headers, response headers with correlation key. Source: same request, Headers tab.
6. VIDEO — Screen recording, about 30 to 45 seconds, full flow. Source: QuickTime File menu, New Screen Recording, or Cmd+Shift+5.
