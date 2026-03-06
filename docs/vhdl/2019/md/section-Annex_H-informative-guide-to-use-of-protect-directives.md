---
title: "Section Annex.H: IEEE Standard for VHDL Language Reference Manual"
document: "VHDL Language Reference Manual"
standard: "IEEE 1076-2019"
domain: "VHDL"
section: "Annex.H"
source_txt: "section-Annex_H-informative-guide-to-use-of-protect-directives.txt"
source_pdf: "/Users/richarddje/Documents/github/VHDL-LRM-IEEE-1076-2019.pdf"
---

# Section Annex.H: IEEE Standard for VHDL Language Reference Manual

IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
597
Copyright © 2019 IEEE. All rights reserved.
Annex H
(informative)
Guide to use of protect directives
H.1 General
The protect tool directives described in Clause 24 allow authors of VHDL descriptions (so called IP) to
provide IP to users in such a way that the users cannot read the source text of the IP. The protect tool
directives provide some underlying mechanisms for such protected IP exchange. This annex provides
guidelines on using the protect tool directives. Note, however, that once IP has been delivered to a user’s
tool, the strength of protection against disclosure of the IP is entirely dependent on the tool.
The protect tool directives are used to form a cryptographic protocol in which IP is sent from the author to
one or more user’s tools, with the users considered untrusted third parties. Cryptographic protocols can be
constructed to support the following use cases, among others:
—
Delivery of IP from an author to any instance of a given decryption tool, and not for use on other
decryption tools
—
Delivery of IP from an author to a specific instance of a given decryption tool, and not for use on
other instances of that decryption tool or any other decryption tool
—
Delivery of IP from an author to a specific user for decryption by any of that user’s decryption tools,
and not for use by other users
—
Delivery of IP from an author to several specific instances of a given decryption tool, and not for use
on other instances of that decryption tool or any other decryption tool
—
Delivery of IP from an author to several specific users for decryption by any of those users’
decryption tools, and not for use by other users
—
Use by a decryption tool of IP delivered by several authors
—
Use by a user of IP delivered by several authors
Central to implementation of these use cases is embedding of appropriate encryption keys in tools. For
example, decryption of IP can be limited to a specific instance of a given tool by embedding a given key in
that instance only. Decryption can be limited to any instance of a given tool by embedding a given key in
each instance, and not in any other tools. Decryption can be limited to a given user by providing that user
with a key to be embedded in the user’s tools.
The way in which keys may be embedded in tools and exchanged among authors, users, and tools is not
specified by this standard. Nonetheless, secure exchange of keys is an integral part of any cryptographic
protocol. This is discussed further in H.5. First, however, follows a discussion of various use cases,
assuming the necessary keys are in place.
H.2 Simple protection envelopes
H.2.1 Symmetric cipher and secret key
The simplest form of IP delivery involves a symmetric cipher using a secret key shared by the IP author and
the decryption tool. The author forms an encryption envelope in which is specified the symmetric cipher and
the secret key to use. For example, the following encryption envelope specifies the AES symmetric cipher
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
598
Copyright © 2019 IEEE. All rights reserved.
using a secret key owned by a given user. Both the encrypting tool and the decrypting tool are assumed to
have access to the secret key.
`protect data_keyowner="ACME IP User", data_method="aes192-cbc"
`protect begin
IP source text
...
`protect end
The encryption tool generates a decryption envelope specifying the cipher and secret key:
`protect begin_protected
`protect encrypt_agent="Encryptomatic", encrypt_agent_fo="2.3.4a"
`protect data_keyowner="ACME IP User", data_method="aes192-cbc"
`protect encoding = (enctype="base64", line_length=40, bytes=4006), data_block
encoded encrypted IP
...
`protect end_protected
The user’s decryption tool uses the key owner information to access the secret key and decrypts the IP using
the AES cipher with that key.
H.2.2 Default cipher and key
The rules for protection envelopes allow specification of the cipher and key to be omitted, in which case, the
cipher and key are chosen in an implementation-defined manner. One possible way for this mechanism to be
used is to imply encryption using a default cipher with a key provided by the tool vendor and embedded in
the encryption and decryption tools. For example, an encryption envelope using this scheme contains only
the directives bracketing the IP source code:
`protect begin
IP source text
...
`protect end
The encryption tool includes information about the cipher and key it chooses in the decryption envelope:
`protect begin_protected
`protect encrypt_agent="Encryptomatic", encrypt_agent_fo="2.3.4a"
`protect data_keyowner="Electrowizz Co", data_keyname="crypto-101"
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
599
Copyright © 2019 IEEE. All rights reserved.
`protect data_method="des-cbc"
`protect encoding = (enctype="base64", line_length=40, bytes=4006), data_block
encoded encrypted IP
...
`protect end_protected
H.2.3 Specification of encoding method
An encryption envelope may also include specification of the encoding method to use for encrypted
information in the decryption envelope produced by the encryption tool. In the absence of an encoding
directive in the encryption envelope, the encryption tool chooses a method, as in the preceding example. An
example including an encoding directive is:
`protect data_keyowner="ACME IP User", data_method="aes192-cbc"
`protect encoding = (enctype="quoted-printable", line_length=60)
`protect begin
IP source text
...
`protect end
H.3 Digital envelopes
H.3.1 Encryption for a single user
A digital envelope allows an author to provide IP to one or more selected tools or users. A common use case
is encryption using an asymmetric cipher for a single user’s decryption tool. The private key is embedded in
the user’s tool, and the public key is published. While the IP could be encrypted using the public key, using
a simple decryption envelope as described in H.2, asymmetric encryption is computationally expensive.
Instead, the author can specify that a digital envelope be used, with a symmetric cipher used to encrypt the
IP, and the key for the symmetric cipher encrypted using the decryption tool’s public key. The encryption
envelope is specified as follows:
`protect key_keyowner="ACME IP User", key_method="rsa", key_block
`protect data_method="aes192-cbc"
`protect begin
IP source text
...
`protect end
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
600
Copyright © 2019 IEEE. All rights reserved.
In this case, the presence of the key keyowner and key method directives specifies that the encryption tool
use a digital envelope. The data method directive specifies the particular symmetric for encrypting the IP.
The encryption tool chooses a session key (that is, the key used to encrypt and decrypt the IP). In the
decryption envelope, it includes a key block containing the encrypted session key and a data block
containing the encrypted IP, as follows:
`protect begin_protected
`protect encrypt_agent="Encryptomatic", encrypt_agent_fo="2.3.4a"
`protect key_keyowner="ACME IP User", key_method="rsa"
`protect encoding = (enctype="base64", line_length=40, bytes=256), key_block
encoded encrypted session key
...
`protect data_method="aes192-cbc"
`protect encoding = (enctype="base64", line_length=40, bytes=4006), data_block
encoded encrypted IP
...
`protect end_protected
The manner in which the encryption tool chooses the session key is implementation defined. It may, for
example, be a default key used for all digital envelopes; however, that would be cryptographically weak. A
better approach is to generate a session key randomly for use in that digital envelope only. Schemes for
generation of random keys are published in the open literature and implemented in widely available software
libraries.
H.3.2 Encryption for multiple users
A variation on the preceding use case allows provision of IP to multiple users’ tools. The IP is encrypted
using a session key and a symmetric cipher, as before, but the session key is encrypted multiple times, once
for each user’s tool. The encryption envelope specifies the users’ keys, as follows:
`protect key_keyowner="ACME IP User1", key_method="rsa", key_block
`protect key_keyowner="ACME IP User2", key_method="elgamal", key_block
`protect key_keyowner="ACME IP User3", key_method="aes192-cbc", key_block
`protect data_method="aes192-cbc"
`protect begin
IP source text
...
`protect end
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
601
Copyright © 2019 IEEE. All rights reserved.
The decryption envelope generated by the encryption tool is:
`protect begin_protected
`protect encrypt_agent="Encryptomatic", encrypt_agent_fo="2.3.4a"
`protect key_keyowner="ACME IP User1", key_method="rsa"
`protect encoding = (enctype="base64", line_length=40, bytes=256)
`protect key_block
encoded encrypted session key
...
`protect key_keyowner="ACME IP User2", key_method="elgamal"
`protect encoding = (enctype="base64", line_length=40, bytes=256)
`protect key_block
encoded encrypted session key
...
`protect key_keyowner="ACME IP User3", key_method="aes192-cbc"
`protect encoding = (enctype="base64", line_length=40, bytes=24)
`protect key_block
encoded encrypted session key
...
`protect data_method="aes192-cbc"
`protect encoding = (enctype="base64", line_length=40, bytes=4006)
`protect data_block
encoded encrypted IP
...
`protect end_protected
Each user’s decryption tool examines the key blocks in the decryption envelope to find one encrypted using
a key to which the tool has access. It then uses that key to decrypt the session key, and then uses the session
key to decrypt the IP.
This example also illustrates a further variation. The cipher used to encrypt a session key need not be an
asymmetric cipher. If a digital envelope is used as a means of providing IP to multiple users, the choice of
cipher and key for session key encryption can be made independently for each user.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
602
Copyright © 2019 IEEE. All rights reserved.
H.4 Digital signatures
A digital signature allows detection of alteration of the IP provided by an author. A scenario in which
alteration might be attempted involves provision of IP to a user, encrypted with the public key of the user’s
tool. A malicious third party may have access to the public key, since it published by the user. The third
party could spoof the IP author, for example, by intercepting the media on which IP is delivered, and provide
a substitute decryption envelope containing malicious IP. The malicious IP would also be encrypted with the
public key of the user’s tool. If the user were unaware of the substitution, he or she would invoke the
decryption tool to decrypt the malicious IP using the tool’s private key. Use of the malicious IP might cause
damage to the user’s business and consequential damage to the IP author.
Scenarios such as this can be avoided by having the IP author sign the IP. Signing involves application of a
hash function to the IP text to compute a digest of the IP. The hash function has the property that application
to different texts produces different digests. Moreover, it is not possible to reconstruct the text from a digest.
The digest is encrypted using the author’s private key and provided along with the IP. The only way the
encrypted digest can be properly decrypted is with the author’s public key, which the author has published.
The user’s tool receiving the IP recomputes the digest using the same hash function on the received IP. The
tool also decrypts the author’s digest using the author’s public key, and compares that digest with the
recomputed digest. If they are the same, the user has confidence that the received IP is unaltered. If they
differ, the delivery has been modified. In that case, the user should not trust the received IP.
The author includes digest directives in the encryption envelope to specify that a digital signature be used.
The digest directives can specify a hash function to use and key for encrypting the digest. If either of these
specifications is omitted, the encryption tool chooses the hash function or key in an implementation-defined
manner. A typical choice would be to use a default hash function or a default key previously identified by
the author. An example encryption envelope specifying a digital signature is:
`protect key_keyowner="ACME IP User", key_method="rsa", key_block
`protect data_method="aes192-cbc"
`protect digest_keyowner="ACME IP Author", digest_key_method="rsa"
`protect digest_method="sha1", digest_block
`protect begin
IP source text
...
`protect end
The decryption envelope produced by the tool is:
`protect begin_protected
`protect key_keyowner="ACME IP User", key_method="rsa"
`protect encoding = (enctype="base64", line_length=40, bytes=256), key_block
encoded encrypted session key
...
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
603
Copyright © 2019 IEEE. All rights reserved.
`protect data_method="aes192-cbc"
`protect encoding = (enctype="base64", line_length=40, bytes=4006), data_block
encoded encrypted IP
...
`protect digest_keyowner="ACME IP Author", digest_key_method="rsa"
`protect digest_method="sha1"
`protect encoding = (enctype="base64", line_length=40, bytes=16), digest_block
encoded encrypted digest
...
`protect end_protected
While this example shows a digital signature used with a digital envelope, that is not a requirement. A digital
signature can augment a simple protection envelope as described in H.2.
H.5 Key exchange
Protection of IP from disclosure relies on security of encryption keys. Should a key become known to an
unauthorized party, the encrypted IP can be decrypted and disseminated. In conventional encryption, the
intended recipient of a message is assumed to have an interest in the security of an encrypted message and is
trusted to keep keys secret. In the context of protected IP exchange, the true recipient is the user’s tool, not
the user. The IP author might not trust the user not to examine or use the IP in unauthorized ways.
Nonetheless, the author must provide the IP to the user’s tools so that the user can gain the benefit of the IP.
Moreover, exchange of keys between the author and the user’s tools may need to be mediated by the user.
These considerations make key exchange more complicated than in many conventional applications of
cryptography.
Many applications that require secure exchange of keys rely on public key infrastructure (PKI). Parties to
communication generate, or are given, key pairs for use with asymmetric ciphers. Each party keeps their
private key secret, and publishes their public key, for example, in a directory. In order to establish that a
public key does, in fact, belong to a given party, the public key is digitally signed by a trusted authority. The
signed public key is represented in the form of a digital certificate, containing the key and the signature. The
trusted authority is called a certification authority (CA). Many PKI systems have a hierarchy of CAs,
allowing a certificate signed by a subordinate CA to be signed by a superior CA, allowing trust to be
distributed hierarchically. One or more root CAs are required to be globally trusted.
Key exchange for IP protection may be built upon public key infrastructure. For example, a vendor of a
decryption tool may embed a private key of a key pair in the tool and register the public key with a CA. The
tool can then generate a key pair for the tool’s user, keeping the private key secret and signing the public key
with both the vendor’s private key and the user’s private key. This allows verification that the public key
originates with the instance of the vendor’s tool owned by the tool user. That public key may then be used by
IP authors to provide IP for that use of that tool only. Similar mechanisms might also be employed within
tools to allow exchange of private keys among tools without disclosure to the tools’ user.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
IEEE Std 1076-2019
IEEE Standard for VHDL Language Reference Manual
604
Copyright © 2019 IEEE. All rights reserved.
In addition to providing for secure key exchange, a decryption tool should take measures to ensure that
stored keys are not disclosed to the tool user (see 24.1.6). If a tool user could read a tool’s stored keys, the
user could decrypt IP independently of the tool.
Authorized licensed use limited to: BOURNEMOUTH UNIVERSITY. Downloaded on December 30,2019 at 14:55:36 UTC from IEEE Xplore.  Restrictions apply.
