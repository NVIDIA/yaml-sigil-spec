# Third-Party Notices

NVIDIA-authored YamlSigil material is licensed under the Apache License 2.0.
The following notices apply only to the identified third-party material.
That material remains subject to its source terms and is not relicensed under
Apache-2.0.

Identification of a source does not imply affiliation with or endorsement by
its authors, publishers, standards organizations, or copyright holders.
YamlSigil is not an IETF RFC, an IRTF publication, or a Standards for
Efficient Cryptography Group (SECG) publication.

## NIST ACVP-Server test data

The `yaml-sigil-spec` repository includes a renamed, otherwise unmodified copy
of the following NIST ACVP-Server file:

- Source repository: <https://github.com/usnistgov/ACVP-Server>.
- Source commit: `15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0`.
- Source path:
  `gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json`.
- Local path:
  `conformance/rebuild-rs/vendor/acvp/ECDSA-SigGen-FIPS186-5.json`.

National Institute of Standards and Technology notice from the pinned
ACVP-Server repository:

> NIST-developed software is provided by NIST as a public service. You may
> use, copy, and distribute copies of the software in any medium, provided
> that you keep intact this entire notice. You may improve, modify, and
> create derivative works of the software or any portion of the software,
> and you may copy and distribute such modifications or works. Modified
> works should carry a notice stating that you changed the software and
> should note the date and nature of any such change. Please explicitly
> acknowledge the National Institute of Standards and Technology as the
> source of the software.
>
> NIST-developed software is expressly provided "AS IS." NIST MAKES NO
> WARRANTY OF ANY KIND, EXPRESS, IMPLIED, IN FACT, OR ARISING BY OPERATION OF
> LAW, INCLUDING, WITHOUT LIMITATION, THE IMPLIED WARRANTY OF
> MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
> DATA ACCURACY. NIST NEITHER REPRESENTS NOR WARRANTS THAT THE OPERATION OF
> THE SOFTWARE WILL BE UNINTERRUPTED OR ERROR-FREE, OR THAT ANY DEFECTS WILL
> BE CORRECTED. NIST DOES NOT WARRANT OR MAKE ANY REPRESENTATIONS REGARDING
> THE USE OF THE SOFTWARE OR THE RESULTS THEREOF, INCLUDING BUT NOT LIMITED
> TO THE CORRECTNESS, ACCURACY, RELIABILITY, OR USEFULNESS OF THE SOFTWARE.
>
> You are solely responsible for determining the appropriateness of using
> and distributing the software and you assume all risks associated with
> its use, including but not limited to the risks and costs of program
> errors, compliance with applicable laws, damage to or loss of data,
> programs or equipment, and the unavailability or interruption of
> operation. This software is not intended to be used in any situation where
> a failure could cause risk of injury or damage to property. The software
> developed by NIST employees is not subject to copyright protection within
> the United States.

The National Institute of Standards and Technology is explicitly acknowledged
as the source of this ACVP test data. The local file name was changed; its
contents were not modified.

The local generator in `conformance/rebuild-rs/src/acvp.rs` and
`conformance/rebuild-rs/src/alg_ecdsa.rs` selects and repackages one record
into
`conformance/alg-ecdsa/acvp-fips186-5-p256-sha256-tc131.binpb` and its
expected-result sidecar. Those derived formats are not NIST publications, and
NIST does not endorse them.

## NIST FIPS 180-4

The ECDSA algorithm specification requires SHA-256 as defined in section 6.2
of:

> National Institute of Standards and Technology (2015), *Secure Hash
> Standard (SHS)*, Federal Information Processing Standards Publication
> FIPS 180-4, <https://doi.org/10.6028/NIST.FIPS.180-4>.

The local uses are in
`algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md` and
`conformance/rebuild-rs/src/alg_ecdsa.rs`. The generator calls the locked
`sha2` crate rather than copying FIPS 180-4's implementation tables. That
crate's automatic notice collection is described below.

## NIST FIPS 186-5

The conformance generator implements ECDSA operations and cites test
requirements from FIPS 186-5:

> National Institute of Standards and Technology (2023), *Digital Signature
> Standard (DSS)*, Federal Information Processing Standards Publication
> FIPS 186-5, <https://doi.org/10.6028/NIST.FIPS.186-5>.

Republished courtesy of the National Institute of Standards and Technology.
NIST technical publications and data are provided as-is, without warranties,
and NIST does not grant patent rights through publication.

The FIPS 186-5 uses are in
`algorithms/02-ECDSA_SECP256R1_SHA256_RAW_RS64.md`,
`conformance/rebuild-rs/src/p256.rs`, and
`conformance/rebuild-rs/src/alg_ecdsa.rs`. NIST's general copyright and
disclaimer policy is <https://www.nist.gov/copyrights-disclaimers>. These
YamlSigil adaptations are not NIST publications and are not endorsed by NIST.

## RFC 8032 test-vector material

The conformance generator, fixtures, and tests use Ed25519 test-vector values
from RFC 8032 section 7.1. RFC 8032 is an IRTF Stream RFC. Section 8(g) of the
IETF Trust Legal Provisions in effect when RFC 8032 was published states that
the provisions for IETF Code Components do not apply to documents in the IRTF
Document Stream. Accordingly, this project does not characterize the section
7.1 values as IETF Code Components or apply the Revised BSD License to them.
They are third-party RFC test-vector material used with attribution under the
applicable BCP 78 and IETF Trust framework.

Copyright (c) 2017 IETF Trust and the persons identified as the document
authors. All rights reserved.

RFC 8032 states that the document is subject to BCP 78 and the IETF Trust's
Legal Provisions Relating to IETF Documents in effect on its publication
date. Section 7(a) of those provisions supplies this warranty disclaimer:

> ALL DOCUMENTS AND THE INFORMATION CONTAINED THEREIN ARE PROVIDED ON AN
> "AS IS" BASIS AND THE CONTRIBUTOR, THE ORGANIZATION HE/SHE REPRESENTS OR
> IS SPONSORED BY (IF ANY), THE INTERNET SOCIETY, THE IETF TRUST, THE
> INTERNET ENGINEERING TASK FORCE AND ANY APPLICABLE MANAGERS OF ALTERNATE
> STREAM DOCUMENTS, AS DEFINED IN SECTION 8 BELOW, DISCLAIM ALL WARRANTIES,
> EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTY THAT THE USE
> OF THE INFORMATION THEREIN WILL NOT INFRINGE ANY RIGHTS OR ANY IMPLIED
> WARRANTIES OF MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.

Source: Simon Josefsson and Ilari Liusvaara, RFC 8032, *Edwards-Curve Digital
Signature Algorithm (EdDSA)*, January 2017:

- RFC information and copyright notice:
  <https://www.rfc-editor.org/info/rfc8032/>.
- Section 7.1 test vectors:
  <https://www.rfc-editor.org/rfc/rfc8032#section-7.1>.
- BCP 78: <https://www.rfc-editor.org/info/bcp78>.
- IETF Trust Legal Provisions, version 5.0:
  <https://trustee.ietf.org/documents/trust-legal-provisions/tlp-5/>.

The names of the document authors, the Crypto Forum Research Group, the IRTF,
the IETF, the IETF Trust, and the RFC Editor are not used to endorse or promote
YamlSigil. No affiliation, sponsorship, or endorsement is claimed or implied.

The algorithm specification and generator also use RFC 8032 sections 5.1,
5.1.2, 5.1.5, 5.1.6, and 5.1.7 for Ed25519 parameters, encodings, signing and
verification procedures, the group order, and the cofactor. Section 3(c) of
the IETF Trust Legal Provisions, version 5.0, addresses reproduction outside
the IETF Standards Process. Section 5(a) states that no patent license is
granted, and sections 7(b) through 7(d) provide the
intellectual-property-rights caveat. RFC test-vector octets remain unchanged;
Rust literals, YAML base64, protobuf framing, sidecars, and YamlSigil state
terminology are identified adaptations.

## RFC 4648 material

The specification and conformance generator use the canonical-encoding rules,
base64url alphabet, and test values from RFC 4648 sections 3, 5, and 10.
RFC 4648 section 15 provides these copying conditions for the abstract and
sections 1, 3, 8, 10, 12, 13, and 14:

> Copyright (c) 2000-2006 Simon Josefsson
>
> Regarding the abstract and sections 1, 3, 8, 10, 12, 13, and 14 of this
> document, that were written by Simon Josefsson ("the author", for the
> remainder of this section), the author makes no guarantees and is not
> responsible for any damage resulting from its use. The author grants
> irrevocable permission to anyone to use, modify, and distribute it in any
> way that does not diminish the rights of anyone else to use, modify, and
> distribute it, provided that redistributed derivative works do not contain
> misleading author or version information and do not falsely purport to be
> IETF RFC documents. Derivative works need not be licensed under similar
> terms.

RFC 4648 also includes this full copyright and warranty statement:

> Copyright (C) The Internet Society (2006).
>
> This document is subject to the rights, licenses and restrictions contained
> in BCP 78, and except as set forth therein, the authors retain all their
> rights.
>
> This document and the information contained herein are provided on an
> "AS IS" basis and THE CONTRIBUTOR, THE ORGANIZATION HE/SHE REPRESENTS OR
> IS SPONSORED BY (IF ANY), THE INTERNET SOCIETY AND THE INTERNET ENGINEERING
> TASK FORCE DISCLAIM ALL WARRANTIES, EXPRESS OR IMPLIED, INCLUDING BUT NOT
> LIMITED TO ANY WARRANTY THAT THE USE OF THE INFORMATION HEREIN WILL NOT
> INFRINGE ANY RIGHTS OR ANY IMPLIED WARRANTIES OF MERCHANTABILITY OR FITNESS
> FOR A PARTICULAR PURPOSE.

This project identifies its derived specification, implementation, and tests
as YamlSigil material and does not represent them as an IETF RFC. It reproduces
only the RFC material needed to explain and test conformance.

Source: Simon Josefsson, RFC 4648, *The Base16, Base32, and Base64 Data
Encodings*, October 2006, <https://www.rfc-editor.org/rfc/rfc4648>.

The RFC's intellectual-property notice states that the IETF takes no position
on the validity or scope of asserted rights or their availability for license,
has made no independent effort to identify such rights, and invites rights
holders to disclose them through the IETF process.

## RFC 3629 UTF-8 material

The `keyid` conformance generator reproduces the UTF-8 octet-count sentence
and four-row encoding table from section 3 of:

> François Yergeau, RFC 3629, *UTF-8, a transformation format of ISO 10646*,
> November 2003, <https://www.rfc-editor.org/rfc/rfc3629.html>.

The local excerpt is in `conformance/rebuild-rs/src/key_id.rs`. It supports
the `U+1F600` 1024-octet and 1028-octet boundary fixtures under
`conformance/key-id/`.

RFC 3629 section 18 states:

> Copyright (C) The Internet Society (2003). All Rights Reserved.
>
> This document and translations of it may be copied and furnished to
> others, and derivative works that comment on or otherwise explain it or
> assist in its implementation may be prepared, copied, published and
> distributed, in whole or in part, without restriction of any kind,
> provided that the above copyright notice and this paragraph are included
> on all such copies and derivative works. However, this document itself may
> not be modified in any way, such as by removing the copyright notice or
> references to the Internet Society or other Internet organizations, except
> as needed for the purpose of developing Internet standards in which case
> the procedures for copyrights defined in the Internet Standards process
> must be followed, or as required to translate it into languages other than
> English.
>
> The limited permissions granted above are perpetual and will not be
> revoked by the Internet Society or its successors or assigns.
>
> This document and the information contained herein is provided on an "AS
> IS" basis and THE INTERNET SOCIETY AND THE INTERNET ENGINEERING TASK FORCE
> DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
> ANY WARRANTY THAT THE USE OF THE INFORMATION HEREIN WILL NOT INFRINGE ANY
> RIGHTS OR ANY IMPLIED WARRANTIES OF MERCHANTABILITY OR FITNESS FOR A
> PARTICULAR PURPOSE.

Section 16 states that the IETF takes no position on the validity or scope of
asserted intellectual-property rights or their availability for license, has
made no effort to identify such rights, and invites rights holders to disclose
them through the IETF process. The local generator is an independently
identified implementation aid and is not represented as an RFC.

## Protocol Buffers documentation

The conformance generator quotes and adapts behavior from the Protocol
Buffers encoding and proto3 language documentation:

- Upstream repository: <https://github.com/protocolbuffers/protocolbuffers.github.io>.
- Pinned repository revision:
  [`881cf1e4cfe0a6bd74ac7d63ceed7a92398e35b7`](https://github.com/protocolbuffers/protocolbuffers.github.io/tree/881cf1e4cfe0a6bd74ac7d63ceed7a92398e35b7).
- Encoding source:
  `content/programming-guides/encoding.md`, including varints, record tags,
  length-delimited records, repeated singular fields, and worked examples.
- Proto3 source:
  `content/programming-guides/proto3.md`, including unknown-field
  preservation and unknown enum values.
- Local files: `conformance/rebuild-rs/src/wire.rs`,
  `conformance/rebuild-rs/src/protobuf_conformance.rs`, and
  `conformance/rebuild-rs/src/schema_alignment.rs`.
- Derived data: `conformance/protobuf-conformance/`,
  `conformance/schema-alignment/`, and every generated `.binpb` fixture that
  uses the shared wire helpers.

The documentation repository supplies the following three-clause BSD license:

```text
Copyright 2021 Google Inc. All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

   * Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.
   * Redistributions in binary form must reproduce the above
copyright notice, this list of conditions and the following disclaimer
in the documentation and/or other materials provided with the
distribution.
   * Neither the name of Google Inc. nor the names of its
contributors may be used to endorse or promote products derived from
this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

Code generated by the Protocol Buffer compiler is owned by the owner
of the input file used when generating it. This code is not
standalone and requires a support library to be linked with it. This
support library is itself covered by the above license.
```

The local `.proto` files are NVIDIA-authored inputs and this repository does
not check in code generated by the Protocol Buffer compiler. The final
paragraph is preserved because it is part of the upstream license, not because
generated Protocol Buffers code is redistributed here. Google, the Protocol
Buffers project, and their contributors do not endorse YamlSigil.

## Standards for Efficient Cryptography

The conformance generator uses elliptic-curve operations and encodings from
*Standards for Efficient Cryptography 1 (SEC 1)*, Version 2.0, and the
secp256r1 domain parameters from *Standards for Efficient Cryptography 2
(SEC 2)*, Version 2.0.

The front page of *Standards for Efficient Cryptography 1 (SEC 1)* carries
this notice:

> Copyright © 2009 Certicom Corp.
>
> License to copy this document is granted provided it is identified as
> "Standards for Efficient Cryptography 1 (SEC 1)", in all material mentioning
> or referencing it.

The front page of *Standards for Efficient Cryptography 2 (SEC 2)* carries
this notice:

> Copyright © 2010 Certicom Corp.
>
> License to copy this document is granted provided it is identified as
> "Standards for Efficient Cryptography 2 (SEC 2)", in all material mentioning
> or referencing it.

Section 1.5, "Intellectual Property," of *Standards for Efficient Cryptography
1 (SEC 1)* states:

> The reader's attention is called to the possibility that compliance with
> this document may require use of an invention covered by patent rights. By
> publication of this document, no position is taken with respect to the
> validity of this claim or of any patent rights in connection therewith. The
> patent holder(s) may have filed with the SECG a statement of willingness to
> grant a license under these rights on reasonable and nondiscriminatory terms
> and conditions to applicants desiring to obtain such a license. Additional
> details may be obtained from the patent holder and from the SECG website,
> <http://www.secg.org>.

Section 1.4, "Intellectual Property," of *Standards for Efficient Cryptography
2 (SEC 2)* states:

> The reader's attention is called to the possibility that compliance with
> this document may require use of an invention covered by patent rights. By
> publication of this document, no position is taken with respect to the
> validity of this claim or of any patent rights in connection therewith. The
> patent holder(s) may have filed with the SECG a statement of willingness to
> grant a license under these rights on fair, reasonable and nondiscriminatory
> terms and conditions to applicants desiring to obtain such a license.
> Additional details may be obtained from the patent holder and from the SECG
> website, <http://www.secg.org>.

Sources:

- *Standards for Efficient Cryptography 1 (SEC 1): Elliptic Curve
  Cryptography*, Version 2.0, May 21, 2009,
  <https://www.secg.org/sec1-v2.pdf>.
- *Standards for Efficient Cryptography 2 (SEC 2): Recommended Elliptic Curve
  Domain Parameters*, Version 2.0, January 27, 2010,
  <https://www.secg.org/sec2-v2.pdf>.

The SEC 1 and SEC 2 material is not relicensed under Apache-2.0. YamlSigil is
not affiliated with, sponsored by, or endorsed by SECG or Certicom Corp.

## Ed25519 small-order point data

The conformance generator uses the eight numeric encodings of the
edwards25519 small-order points reported in Table 1 and Appendix B of:

> Konstantinos Chalkias, Francois Garillot, and Valeria Nikolaenko, *Taming
> the Many EdDSAs*, IACR Cryptology ePrint Archive, Report 2020/1244,
> <https://eprint.iacr.org/2020/1244>.

The ePrint file is licensed under the Creative Commons Attribution 4.0
International license (CC BY 4.0):
<https://creativecommons.org/licenses/by/4.0/>. This project extracts the eight
numeric encodings from Table 1 and Appendix B, removes whitespace and table
formatting, lowercases and reorders the values, represents them as hexadecimal
Rust string literals, and emits them as a newline-delimited conformance
fixture. The numeric values are unchanged. The algorithm specification also
adapts Algorithm 2 into YamlSigil terminology and failure mappings. No
endorsement by the authors or IACR is implied.

The complete CC BY 4.0 legal code follows.

```text
Attribution 4.0 International

=======================================================================

Creative Commons Corporation ("Creative Commons") is not a law firm and
does not provide legal services or legal advice. Distribution of
Creative Commons public licenses does not create a lawyer-client or
other relationship. Creative Commons makes its licenses and related
information available on an "as-is" basis. Creative Commons gives no
warranties regarding its licenses, any material licensed under their
terms and conditions, or any related information. Creative Commons
disclaims all liability for damages resulting from their use to the
fullest extent possible.

Using Creative Commons Public Licenses

Creative Commons public licenses provide a standard set of terms and
conditions that creators and other rights holders may use to share
original works of authorship and other material subject to copyright
and certain other rights specified in the public license below. The
following considerations are for informational purposes only, are not
exhaustive, and do not form part of our licenses.

     Considerations for licensors: Our public licenses are
     intended for use by those authorized to give the public
     permission to use material in ways otherwise restricted by
     copyright and certain other rights. Our licenses are
     irrevocable. Licensors should read and understand the terms
     and conditions of the license they choose before applying it.
     Licensors should also secure all rights necessary before
     applying our licenses so that the public can reuse the
     material as expected. Licensors should clearly mark any
     material not subject to the license. This includes other CC-
     licensed material, or material used under an exception or
     limitation to copyright. More considerations for licensors:
    wiki.creativecommons.org/Considerations_for_licensors

     Considerations for the public: By using one of our public
     licenses, a licensor grants the public permission to use the
     licensed material under specified terms and conditions. If
     the licensor's permission is not necessary for any reason--for
     example, because of any applicable exception or limitation to
     copyright--then that use is not regulated by the license. Our
     licenses grant only permissions under copyright and certain
     other rights that a licensor has authority to grant. Use of
     the licensed material may still be restricted for other
     reasons, including because others have copyright or other
     rights in the material. A licensor may make special requests,
     such as asking that all changes be marked or described.
     Although not required by our licenses, you are encouraged to
     respect those requests where reasonable. More considerations
     for the public:
    wiki.creativecommons.org/Considerations_for_licensees

=======================================================================

Creative Commons Attribution 4.0 International Public License

By exercising the Licensed Rights (defined below), You accept and agree
to be bound by the terms and conditions of this Creative Commons
Attribution 4.0 International Public License ("Public License"). To the
extent this Public License may be interpreted as a contract, You are
granted the Licensed Rights in consideration of Your acceptance of
these terms and conditions, and the Licensor grants You such rights in
consideration of benefits the Licensor receives from making the
Licensed Material available under these terms and conditions.


Section 1 -- Definitions.

  a. Adapted Material means material subject to Copyright and Similar
     Rights that is derived from or based upon the Licensed Material
     and in which the Licensed Material is translated, altered,
     arranged, transformed, or otherwise modified in a manner requiring
     permission under the Copyright and Similar Rights held by the
     Licensor. For purposes of this Public License, where the Licensed
     Material is a musical work, performance, or sound recording,
     Adapted Material is always produced where the Licensed Material is
     synched in timed relation with a moving image.

  b. Adapter's License means the license You apply to Your Copyright
     and Similar Rights in Your contributions to Adapted Material in
     accordance with the terms and conditions of this Public License.

  c. Copyright and Similar Rights means copyright and/or similar rights
     closely related to copyright including, without limitation,
     performance, broadcast, sound recording, and Sui Generis Database
     Rights, without regard to how the rights are labeled or
     categorized. For purposes of this Public License, the rights
     specified in Section 2(b)(1)-(2) are not Copyright and Similar
     Rights.

  d. Effective Technological Measures means those measures that, in the
     absence of proper authority, may not be circumvented under laws
     fulfilling obligations under Article 11 of the WIPO Copyright
     Treaty adopted on December 20, 1996, and/or similar international
     agreements.

  e. Exceptions and Limitations means fair use, fair dealing, and/or
     any other exception or limitation to Copyright and Similar Rights
     that applies to Your use of the Licensed Material.

  f. Licensed Material means the artistic or literary work, database,
     or other material to which the Licensor applied this Public
     License.

  g. Licensed Rights means the rights granted to You subject to the
     terms and conditions of this Public License, which are limited to
     all Copyright and Similar Rights that apply to Your use of the
     Licensed Material and that the Licensor has authority to license.

  h. Licensor means the individual(s) or entity(ies) granting rights
     under this Public License.

  i. Share means to provide material to the public by any means or
     process that requires permission under the Licensed Rights, such
     as reproduction, public display, public performance, distribution,
     dissemination, communication, or importation, and to make material
     available to the public including in ways that members of the
     public may access the material from a place and at a time
     individually chosen by them.

  j. Sui Generis Database Rights means rights other than copyright
     resulting from Directive 96/9/EC of the European Parliament and of
     the Council of 11 March 1996 on the legal protection of databases,
     as amended and/or succeeded, as well as other essentially
     equivalent rights anywhere in the world.

  k. You means the individual or entity exercising the Licensed Rights
     under this Public License. Your has a corresponding meaning.


Section 2 -- Scope.

  a. License grant.

       1. Subject to the terms and conditions of this Public License,
          the Licensor hereby grants You a worldwide, royalty-free,
          non-sublicensable, non-exclusive, irrevocable license to
          exercise the Licensed Rights in the Licensed Material to:

            a. reproduce and Share the Licensed Material, in whole or
               in part; and

            b. produce, reproduce, and Share Adapted Material.

       2. Exceptions and Limitations. For the avoidance of doubt, where
          Exceptions and Limitations apply to Your use, this Public
          License does not apply, and You do not need to comply with
          its terms and conditions.

       3. Term. The term of this Public License is specified in Section
          6(a).

       4. Media and formats; technical modifications allowed. The
          Licensor authorizes You to exercise the Licensed Rights in
          all media and formats whether now known or hereafter created,
          and to make technical modifications necessary to do so. The
          Licensor waives and/or agrees not to assert any right or
          authority to forbid You from making technical modifications
          necessary to exercise the Licensed Rights, including
          technical modifications necessary to circumvent Effective
          Technological Measures. For purposes of this Public License,
          simply making modifications authorized by this Section 2(a)
          (4) never produces Adapted Material.

       5. Downstream recipients.

            a. Offer from the Licensor -- Licensed Material. Every
               recipient of the Licensed Material automatically
               receives an offer from the Licensor to exercise the
               Licensed Rights under the terms and conditions of this
               Public License.

            b. No downstream restrictions. You may not offer or impose
               any additional or different terms or conditions on, or
               apply any Effective Technological Measures to, the
               Licensed Material if doing so restricts exercise of the
               Licensed Rights by any recipient of the Licensed
               Material.

       6. No endorsement. Nothing in this Public License constitutes or
          may be construed as permission to assert or imply that You
          are, or that Your use of the Licensed Material is, connected
          with, or sponsored, endorsed, or granted official status by,
          the Licensor or others designated to receive attribution as
          provided in Section 3(a)(1)(A)(i).

  b. Other rights.

       1. Moral rights, such as the right of integrity, are not
          licensed under this Public License, nor are publicity,
          privacy, and/or other similar personality rights; however, to
          the extent possible, the Licensor waives and/or agrees not to
          assert any such rights held by the Licensor to the limited
          extent necessary to allow You to exercise the Licensed
          Rights, but not otherwise.

       2. Patent and trademark rights are not licensed under this
          Public License.

       3. To the extent possible, the Licensor waives any right to
          collect royalties from You for the exercise of the Licensed
          Rights, whether directly or through a collecting society
          under any voluntary or waivable statutory or compulsory
          licensing scheme. In all other cases the Licensor expressly
          reserves any right to collect such royalties.


Section 3 -- License Conditions.

Your exercise of the Licensed Rights is expressly made subject to the
following conditions.

  a. Attribution.

       1. If You Share the Licensed Material (including in modified
          form), You must:

            a. retain the following if it is supplied by the Licensor
               with the Licensed Material:

                 i. identification of the creator(s) of the Licensed
                    Material and any others designated to receive
                    attribution, in any reasonable manner requested by
                    the Licensor (including by pseudonym if
                    designated);

                ii. a copyright notice;

               iii. a notice that refers to this Public License;

                iv. a notice that refers to the disclaimer of
                    warranties;

                 v. a URI or hyperlink to the Licensed Material to the
                    extent reasonably practicable;

            b. indicate if You modified the Licensed Material and
               retain an indication of any previous modifications; and

            c. indicate the Licensed Material is licensed under this
               Public License, and include the text of, or the URI or
               hyperlink to, this Public License.

       2. You may satisfy the conditions in Section 3(a)(1) in any
          reasonable manner based on the medium, means, and context in
          which You Share the Licensed Material. For example, it may be
          reasonable to satisfy the conditions by providing a URI or
          hyperlink to a resource that includes the required
          information.

       3. If requested by the Licensor, You must remove any of the
          information required by Section 3(a)(1)(A) to the extent
          reasonably practicable.

       4. If You Share Adapted Material You produce, the Adapter's
          License You apply must not prevent recipients of the Adapted
          Material from complying with this Public License.


Section 4 -- Sui Generis Database Rights.

Where the Licensed Rights include Sui Generis Database Rights that
apply to Your use of the Licensed Material:

  a. for the avoidance of doubt, Section 2(a)(1) grants You the right
     to extract, reuse, reproduce, and Share all or a substantial
     portion of the contents of the database;

  b. if You include all or a substantial portion of the database
     contents in a database in which You have Sui Generis Database
     Rights, then the database in which You have Sui Generis Database
     Rights (but not its individual contents) is Adapted Material; and

  c. You must comply with the conditions in Section 3(a) if You Share
     all or a substantial portion of the contents of the database.

For the avoidance of doubt, this Section 4 supplements and does not
replace Your obligations under this Public License where the Licensed
Rights include other Copyright and Similar Rights.


Section 5 -- Disclaimer of Warranties and Limitation of Liability.

  a. UNLESS OTHERWISE SEPARATELY UNDERTAKEN BY THE LICENSOR, TO THE
     EXTENT POSSIBLE, THE LICENSOR OFFERS THE LICENSED MATERIAL AS-IS
     AND AS-AVAILABLE, AND MAKES NO REPRESENTATIONS OR WARRANTIES OF
     ANY KIND CONCERNING THE LICENSED MATERIAL, WHETHER EXPRESS,
     IMPLIED, STATUTORY, OR OTHER. THIS INCLUDES, WITHOUT LIMITATION,
     WARRANTIES OF TITLE, MERCHANTABILITY, FITNESS FOR A PARTICULAR
     PURPOSE, NON-INFRINGEMENT, ABSENCE OF LATENT OR OTHER DEFECTS,
     ACCURACY, OR THE PRESENCE OR ABSENCE OF ERRORS, WHETHER OR NOT
     KNOWN OR DISCOVERABLE. WHERE DISCLAIMERS OF WARRANTIES ARE NOT
     ALLOWED IN FULL OR IN PART, THIS DISCLAIMER MAY NOT APPLY TO YOU.

  b. TO THE EXTENT POSSIBLE, IN NO EVENT WILL THE LICENSOR BE LIABLE
     TO YOU ON ANY LEGAL THEORY (INCLUDING, WITHOUT LIMITATION,
     NEGLIGENCE) OR OTHERWISE FOR ANY DIRECT, SPECIAL, INDIRECT,
     INCIDENTAL, CONSEQUENTIAL, PUNITIVE, EXEMPLARY, OR OTHER LOSSES,
     COSTS, EXPENSES, OR DAMAGES ARISING OUT OF THIS PUBLIC LICENSE OR
     USE OF THE LICENSED MATERIAL, EVEN IF THE LICENSOR HAS BEEN
     ADVISED OF THE POSSIBILITY OF SUCH LOSSES, COSTS, EXPENSES, OR
     DAMAGES. WHERE A LIMITATION OF LIABILITY IS NOT ALLOWED IN FULL OR
     IN PART, THIS LIMITATION MAY NOT APPLY TO YOU.

  c. The disclaimer of warranties and limitation of liability provided
     above shall be interpreted in a manner that, to the extent
     possible, most closely approximates an absolute disclaimer and
     waiver of all liability.


Section 6 -- Term and Termination.

  a. This Public License applies for the term of the Copyright and
     Similar Rights licensed here. However, if You fail to comply with
     this Public License, then Your rights under this Public License
     terminate automatically.

  b. Where Your right to use the Licensed Material has terminated under
     Section 6(a), it reinstates:

       1. automatically as of the date the violation is cured, provided
          it is cured within 30 days of Your discovery of the
          violation; or

       2. upon express reinstatement by the Licensor.

     For the avoidance of doubt, this Section 6(b) does not affect any
     right the Licensor may have to seek remedies for Your violations
     of this Public License.

  c. For the avoidance of doubt, the Licensor may also offer the
     Licensed Material under separate terms or conditions or stop
     distributing the Licensed Material at any time; however, doing so
     will not terminate this Public License.

  d. Sections 1, 5, 6, 7, and 8 survive termination of this Public
     License.


Section 7 -- Other Terms and Conditions.

  a. The Licensor shall not be bound by any additional or different
     terms or conditions communicated by You unless expressly agreed.

  b. Any arrangements, understandings, or agreements regarding the
     Licensed Material not stated herein are separate from and
     independent of the terms and conditions of this Public License.


Section 8 -- Interpretation.

  a. For the avoidance of doubt, this Public License does not, and
     shall not be interpreted to, reduce, limit, restrict, or impose
     conditions on any use of the Licensed Material that could lawfully
     be made without permission under this Public License.

  b. To the extent possible, if any provision of this Public License is
     deemed unenforceable, it shall be automatically reformed to the
     minimum extent necessary to make it enforceable. If the provision
     cannot be reformed, it shall be severed from this Public License
     without affecting the enforceability of the remaining terms and
     conditions.

  c. No term or condition of this Public License will be waived and no
     failure to comply consented to unless expressly agreed to by the
     Licensor.

  d. Nothing in this Public License constitutes or may be interpreted
     as a limitation upon, or waiver of, any privileges and immunities
     that apply to the Licensor or You, including from the legal
     processes of any jurisdiction or authority.


=======================================================================

Creative Commons is not a party to its public
licenses. Notwithstanding, Creative Commons may elect to apply one of
its public licenses to material it publishes and in those instances
will be considered the “Licensor.” The text of the Creative Commons
public licenses is dedicated to the public domain under the CC0 Public
Domain Dedication. Except for the limited purpose of indicating that
material is shared under a Creative Commons public license or as
otherwise permitted by the Creative Commons policies published at
creativecommons.org/policies, Creative Commons does not authorize the
use of the trademark "Creative Commons" or any other trademark or logo
of Creative Commons without its prior written consent including,
without limitation, in connection with any unauthorized modifications
to any of its public licenses or any other arrangements,
understandings, or agreements concerning use of licensed material. For
the avoidance of doubt, this paragraph does not form part of the
public licenses.

Creative Commons may be contacted at creativecommons.org.

```

## Rust crates in the conformance rebuilder

`conformance/rebuild-rs/Cargo.lock` pins the complete external Cargo dependency
graph used to build the conformance rebuilder. The repository does not vendor
those crate sources. During an image build, Cargo downloads the exact locked
versions and the Dockerfile automatically collects every top-level
`LICENSE*`, `COPYRIGHT*`, and `NOTICE*` file supplied by those crates. The
collected files travel with the binary under:

```text
/usr/share/doc/yamlsigil-conformance-rebuild/cargo/<crate-version>/
```

This avoids maintaining a hand-copied dependency-license inventory in this
file. A `Cargo.lock` change changes the collected set automatically and still
requires review before an image is distributed.

## Rust standard library in the container image

The container builds with `rust:1.95.0-trixie`. The inspected toolchain
reports Rust `1.95.0`, source commit
`59807616e1fa2540724bfbac14d7976d7e4a3860`, dated April 14, 2026. Rust
standard-library code is statically linked into `rebuild_all`.

Rust is primarily distributed under the MIT and Apache-2.0 licenses, with
additional third-party terms applying to identified library components. The
Rust project maintains its license explanation at
<https://www.rust-lang.org/policies/licenses> and its source at
<https://github.com/rust-lang/rust>.

The Docker build copies the pinned toolchain's complete
`share/doc/rust/COPYRIGHT-library.html` into the final image at:

```text
/usr/share/doc/yamlsigil-conformance-rebuild/rust/COPYRIGHT-library.html
```

That upstream-generated file is the authoritative component-by-component
copyright and license inventory for the exact Rust standard library linked for
the image's target platform. It includes the full applicable license texts.
The image also carries this file and the repository `LICENSE` in
`/usr/share/doc/yamlsigil-conformance-rebuild/`.

## Debian runtime packages in the container image

The runtime stage uses `debian:trixie-slim` and installs
`ca-certificates`. The final binary dynamically uses runtime libraries
provided by the Debian image. Debian packages automatically install their
copyright and license records in:

```text
/usr/share/doc/<package>/copyright
```

The Dockerfile preserves those package-supplied files. It does not generate,
copy, enumerate, or require manual maintenance of Debian notices in this
repository.
