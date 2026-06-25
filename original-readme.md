# policy document signing considerations

> Working Draft for format definition is defined in [README.md](./README.md)

## What do we want?

Required:

- Documents need to be signed.
- Users should just be able to confirm a signature, with syntax
  highlighting.
- Auditors, can always see a signed document, based on a "Day-to-day"
  users's simple view of the document.
- Authors, shouldn't have to care about Signatures.

Not Required:

- Day-to-day users don't need to view or deal with the signatures.
  - YAML is likely the preferred view for structured documents.
  - They should be able to trust that they were signed properly, if they
    bother to look at them.
  - None of this suggests that they can't, just that it's okay if it's harder
    to get to more concrete details.

Desired:

- We want "single signed document" that is plaintext and structured cleanly.
  - We are okay with a secondary signature metadata blob if required and cheap.
  - Secondary signature metadata (`.sig` files) are a trivial and known pattern
    but not really a great user experience.

## research of interest

### CNCF

In most cases, signatures are stored "to the side" of the document. All of these
are great for containers. Still searching to see what they offer for YAML, that
might not be to the side, and isn't bolted to 1 language.

- <https://dlorenc.medium.com/notary-v2-and-cosign-b816658f044d>
  - Good notes on weakenesses for `cosign` & `notary` projects
- <https://docs.sigstore.dev/policy-controller/overview/>
  - <https://github.com/sigstore/sigstore-rs>

### protobuf (non-deterministic deserialization)

- cheap with serialized protobufs, but requires more constraints than
  a YAML format would
- cheaper than YAML/JSON, with stronger cross language support for a separated field.
- can be decomposed without sig to YAML easily
- <https://web.tecnico.ulisboa.pt/miguel.pardal/www/pubs/2021_Francisco_Pardal_INForum_POSE.pdf>
  - I like the unsecured headers aspect, that would allow us to offer routing or
    useful information without deserialization.

### serialized but good and adopted

DSSE:

- <https://github.com/secure-systems-lab/dsse>
  - I'm kind of **meh** on DSSE as a user.
  - As a developer, it's great.

## research of less interest

YAML/JSON in-document signing will not provide us with deterministic results
across all platforms without effort.

### yaml

- <https://github.com/sigstore/cosign>
  - not in-toto

### json

- <https://datatracker.ietf.org/doc/html/rfc7515>
  - imo, this is way too complicated.
- JSON Web Signature (JWS) / JWS Compact Serialization
- [COSE](https://datatracker.ietf.org/doc/rfc9052/)
