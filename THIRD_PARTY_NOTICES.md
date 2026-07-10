# Third-Party Notices

YamlSigil is licensed under the Apache License 2.0. The following notices
apply only to the third-party material identified below and do not change the
license for NVIDIA-authored material.

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

## NIST FIPS 186-5

The conformance generator implements ECDSA operations and cites test
requirements from FIPS 186-5:

> National Institute of Standards and Technology (2023), *Digital Signature
> Standard (DSS)*, Federal Information Processing Standards Publication
> FIPS 186-5, <https://doi.org/10.6028/NIST.FIPS.186-5>.

Republished courtesy of the National Institute of Standards and Technology.
NIST technical publications and data are provided as-is, without warranties,
and NIST does not grant patent rights through publication.

## RFC 8032 Code Components

The conformance generator, fixtures, and tests use Ed25519 test-vector values
from RFC 8032 section 7.1. These values are treated as IETF Code Components
under the Revised BSD License in the IETF Trust Legal Provisions.

Copyright (c) 2017 IETF Trust and the persons identified as authors of the
code. All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

- Redistributions of source code must retain the above copyright notice,
  this list of conditions and the following disclaimer.
- Redistributions in binary form must reproduce the above copyright notice,
  this list of conditions and the following disclaimer in the documentation
  and/or other materials provided with the distribution.
- Neither the name of Internet Society, IETF or IETF Trust, nor the names of
  specific contributors, may be used to endorse or promote products derived
  from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.

Source: Simon Josefsson and Ilari Liusvaara, RFC 8032, *Edwards-Curve Digital
Signature Algorithm (EdDSA)*, January 2017,
<https://www.rfc-editor.org/rfc/rfc8032>.

## RFC 4648 material

The conformance generator uses the base64url alphabet and test values from
RFC 4648 sections 5 and 10. RFC 4648 section 15 provides these copying
conditions for the abstract and sections 1, 3, 8, 10, 12, 13, and 14:

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

RFC 4648 is Copyright (C) The Internet Society (2006). The document and its
information are provided on an as-is basis without warranties. This project
identifies its derived implementation and tests as YamlSigil material and does
not represent them as an IETF RFC.

Source: Simon Josefsson, RFC 4648, *The Base16, Base32, and Base64 Data
Encodings*, October 2006, <https://www.rfc-editor.org/rfc/rfc4648>.

## Standards for Efficient Cryptography

The conformance generator uses elliptic-curve operations and encodings from
*Standards for Efficient Cryptography 1 (SEC 1)*, Version 2.0, and the
secp256r1 domain parameters from *Standards for Efficient Cryptography 2
(SEC 2)*, Version 2.0.

Copyright (c) 2009 Certicom Corp. License to copy the SEC 1 document is
granted provided it is identified as "Standards for Efficient Cryptography 1
(SEC 1)" in all material mentioning or referencing it.

Copyright (c) 2010 Certicom Corp. License to copy the SEC 2 document is
granted provided it is identified as "Standards for Efficient Cryptography 2
(SEC 2)" in all material mentioning or referencing it.

Sources:

- *Standards for Efficient Cryptography 1 (SEC 1): Elliptic Curve
  Cryptography*, Version 2.0, <https://www.secg.org/sec1-v2.pdf>.
- *Standards for Efficient Cryptography 2 (SEC 2): Recommended Elliptic Curve
  Domain Parameters*, Version 2.0, <https://www.secg.org/sec2-v2.pdf>.

## Ed25519 small-order point data

The conformance generator uses the eight numeric encodings of the
edwards25519 small-order points reported in Table 5 of:

> Konstantinos Chalkias, Francois Garillot, and Valeria Nikolaenko, *Taming
> the Many EdDSAs*, IACR Cryptology ePrint Archive, Report 2020/1244,
> <https://eprint.iacr.org/2020/1244>.

The numeric encodings are included as factual cryptographic conformance data.
No paper text, table formatting, figures, or other expressive content is
redistributed.
