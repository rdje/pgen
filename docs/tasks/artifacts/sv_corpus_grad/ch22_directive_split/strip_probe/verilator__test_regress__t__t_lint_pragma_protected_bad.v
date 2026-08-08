// DESCRIPTION: Verilator: Verilog Test module
//
// This file ONLY is placed under the Creative Commons Public Domain.
// SPDX-FileCopyrightText: 2021 Wilson Snyder
// SPDX-License-Identifier: CC0-1.0

module t_lint_pragma_protected_err;

// This part should see some failures



// should fail because value should be quoted

// should fail because no value given at all






// expect error in key_block below, 64 bytes but expecting  65
// also expect "multiple `pragma encoding sections` error because number of
// bytes does not go down to 0 in the end of the section below due to the 64->65 change


ICAgICAgICAgICAgICAgICAgIEdOVSBMRVNTRVIgR0VORVJBTCBQVUJMSUMgTElDRU5TRQogICAg
KSAyMDA3IE==






IEV2ZXJ5b25lIGlzIHBlcm1pdHRlZCB0byBjb3B5IGFuZCBkaXN0cmlidXRlIHZlcmJhdGltIGNv
cGllcwogb2YgdGhpcyBsaWNlbnNlIGRvY3VtZW50LCBidXQgY2hhbmdpbmcgaXQgaXMgbm90IGFs
bG93ZWQuCgoKICBUaGl=






TGljZW5zZSBpbmNvcnBvcmF0ZXMKdGhlIHRlcm1zIGFuZCBjb25kaXRpb25zIG9mIHZlcnNpb24g
MyBvZiB0aGUgR05VIEdlbmVyYWwgUHVibGljCkxpY2Vuc2UsIHN1cHBsZW1lbnRlZCBieSB0aGUg
YWRkaXRpb25hbCBwZXJ=



aW5pdGlvbnMuCgogIEFzIHVzZWQgaGVyZWluLCAidGhpcyBMaWNlbnNlIiByZWZlcnMgdG8gdmVy
c2lvbiAzIG9mIHRoZSBHTlUgTGVzc2VyCkdlbmVyYWwgUHVibGljIExpY2Vuc2UsIGFuZCB0aGUg
IkdOVSBHUEwiIHJlZmVycyB0byB2ZXJzaW9uIDMgb2YgdGhlIEdOVQpHZW5lcmFsIFB1YmxpYyBM
aWNlbnNlLgoKICAiVGhlIExpYnJhcnkiIHJlZmVycyB0byBhIGNvdmVyZWQgd29yayBnb3Zlcm5l
ZCBieSB0aGlzIExpY2Vuc2UsCm90aGVyIHRoYW4gYW4gQXBwbGljYXRpb24gb3IgYSBDb21iaW5l
ZCBXb3JrIGFzIG==



aW5pdGlvbnMuCgogIEFzIHVzZWQgaGVyZWluLCAidGhpcyBMaWNlbnNlIiByZWZlcnMgdG8gdmVy
c2lvbiAzIG9mIHRoZSBHTlUgTGVzc2VyCkdlbmVyYWwgUHVibGljIExpY2Vuc2UsIGFuZCB0aGUg
IkdOVSBHUEwiIHJlZmVycyB0byB2ZXJzaW9uIDMgb2YgdGhlIEdOVQpHZW5lcmFsIFB1YmxpYyBM
aWNlbnNlLgoKICAiVGhlIExpYnJhcnkiIHJlZmVycyB0byBhIGNvdmVyZWQgd29yayBnb3Zlcm5l
ZCBieSB0aGlzIExpY2Vuc2UsCm90aGVyIHRoYW4gYW4gQXBwbGljYXRpb24gb3IgYSBDb21iaW5l
ZCBXb3JrIGFzIG==




aW5pdGlvbnMuCgogIEFzIHVzZWQgaGVyZWluLCAidGhpcyBMaWNlbnNlIiByZWZlcnMgdG8gdmVyTOOLONG




aW5p




aW5p



aW5p



aW5p




// Should trigger unknown pragma warning, although in principle unknown pragmas should be safely ignored.


// Should trigger missing pragma warning


endmodule
