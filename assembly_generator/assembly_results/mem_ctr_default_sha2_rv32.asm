
inline_benchmark:	file format elf32-littleriscv

Disassembly of section .text:

00011178 <_start>:
   11178: ec010113     	addi	sp, sp, -0x140
   1117c: 12112e23     	sw	ra, 0x13c(sp)
   11180: 12812c23     	sw	s0, 0x138(sp)
   11184: 12912a23     	sw	s1, 0x134(sp)
   11188: 13212823     	sw	s2, 0x130(sp)
   1118c: 13312623     	sw	s3, 0x12c(sp)
   11190: 03010413     	addi	s0, sp, 0x30
   11194: 04100613     	li	a2, 0x41
   11198: 00040513     	mv	a0, s0
   1119c: 00000593     	li	a1, 0x0
   111a0: 00004097     	auipc	ra, 0x4
   111a4: 574080e7     	jalr	0x574(ra) <memset>
   111a8: 6a09e537     	lui	a0, 0x6a09e
   111ac: bb67b5b7     	lui	a1, 0xbb67b
   111b0: 3c6ef637     	lui	a2, 0x3c6ef
   111b4: a54ff6b7     	lui	a3, 0xa54ff
   111b8: 510e5737     	lui	a4, 0x510e5
   111bc: 9b0577b7     	lui	a5, 0x9b057
   111c0: 1f83e837     	lui	a6, 0x1f83e
   111c4: 66750513     	addi	a0, a0, 0x667
   111c8: e8558593     	addi	a1, a1, -0x17b
   111cc: 37260613     	addi	a2, a2, 0x372
   111d0: 53a68693     	addi	a3, a3, 0x53a
   111d4: 00a12423     	sw	a0, 0x8(sp)
   111d8: 00b12623     	sw	a1, 0xc(sp)
   111dc: 00c12823     	sw	a2, 0x10(sp)
   111e0: 00d12a23     	sw	a3, 0x14(sp)
   111e4: 5be0d537     	lui	a0, 0x5be0d
   111e8: 27f70593     	addi	a1, a4, 0x27f
   111ec: 88c78613     	addi	a2, a5, -0x774
   111f0: 9ab80693     	addi	a3, a6, -0x655
   111f4: d1950513     	addi	a0, a0, -0x2e7
   111f8: 00b12c23     	sw	a1, 0x18(sp)
   111fc: 00c12e23     	sw	a2, 0x1c(sp)
   11200: 02d12023     	sw	a3, 0x20(sp)
   11204: 02a12223     	sw	a0, 0x24(sp)
   11208: 02012423     	sw	zero, 0x28(sp)
   1120c: 02012623     	sw	zero, 0x2c(sp)
   11210: 000105b7     	lui	a1, 0x10
   11214: 0d458593     	addi	a1, a1, 0xd4
   11218: 00b00613     	li	a2, 0xb
   1121c: 00b00493     	li	s1, 0xb
   11220: 00040513     	mv	a0, s0
   11224: 00004097     	auipc	ra, 0x4
   11228: 2ac080e7     	jalr	0x2ac(ra) <memcpy>
   1122c: 06910823     	sb	s1, 0x70(sp)
   11230: 07810513     	addi	a0, sp, 0x78
   11234: 00810593     	addi	a1, sp, 0x8
   11238: 07000613     	li	a2, 0x70
   1123c: 00004097     	auipc	ra, 0x4
   11240: 294080e7     	jalr	0x294(ra) <memcpy>
   11244: 0a010413     	addi	s0, sp, 0xa0
   11248: 0e014483     	lbu	s1, 0xe0(sp)
   1124c: 09812503     	lw	a0, 0x98(sp)
   11250: 09c12583     	lw	a1, 0x9c(sp)
   11254: 00010637     	lui	a2, 0x10
   11258: 08000693     	li	a3, 0x80
   1125c: f0060613     	addi	a2, a2, -0x100
   11260: 00959713     	slli	a4, a1, 0x9
   11264: 00951793     	slli	a5, a0, 0x9
   11268: 00349813     	slli	a6, s1, 0x3
   1126c: 00151893     	slli	a7, a0, 0x1
   11270: 00159593     	slli	a1, a1, 0x1
   11274: 0107e833     	or	a6, a5, a6
   11278: 00c8f8b3     	and	a7, a7, a2
   1127c: 0187d793     	srli	a5, a5, 0x18
   11280: 00c5f5b3     	and	a1, a1, a2
   11284: 00f8e7b3     	or	a5, a7, a5
   11288: 01875893     	srli	a7, a4, 0x18
   1128c: 0115e5b3     	or	a1, a1, a7
   11290: 03f00893     	li	a7, 0x3f
   11294: 01755293     	srli	t0, a0, 0x17
   11298: 00576533     	or	a0, a4, t0
   1129c: 00c87733     	and	a4, a6, a2
   112a0: 00c57633     	and	a2, a0, a2
   112a4: 01b49513     	slli	a0, s1, 0x1b
   112a8: 00f567b3     	or	a5, a0, a5
   112ac: 00940533     	add	a0, s0, s1
   112b0: 01829293     	slli	t0, t0, 0x18
   112b4: 00871713     	slli	a4, a4, 0x8
   112b8: 00861613     	slli	a2, a2, 0x8
   112bc: 00b2e5b3     	or	a1, t0, a1
   112c0: 00e7e9b3     	or	s3, a5, a4
   112c4: 00c5e933     	or	s2, a1, a2
   112c8: 00d50023     	sb	a3, 0x0(a0)
   112cc: 03148263     	beq	s1, a7, 0x112f0 <_start+0x178>
   112d0: 00150513     	addi	a0, a0, 0x1
   112d4: 03f4c613     	xori	a2, s1, 0x3f
   112d8: 00000593     	li	a1, 0x0
   112dc: 00004097     	auipc	ra, 0x4
   112e0: 438080e7     	jalr	0x438(ra) <memset>
   112e4: 0384c513     	xori	a0, s1, 0x38
   112e8: 00700593     	li	a1, 0x7
   112ec: 06a5e663     	bltu	a1, a0, 0x11358 <_start+0x1e0>
   112f0: 07810513     	addi	a0, sp, 0x78
   112f4: 00040593     	mv	a1, s0
   112f8: 00000097     	auipc	ra, 0x0
   112fc: 1e8080e7     	jalr	0x1e8(ra) <sha2::sha256::compress256::h442137f2ea059990>
   11300: 0ec10513     	addi	a0, sp, 0xec
   11304: 03800613     	li	a2, 0x38
   11308: 00000593     	li	a1, 0x0
   1130c: 00004097     	auipc	ra, 0x4
   11310: 408080e7     	jalr	0x408(ra) <memset>
   11314: 0189d513     	srli	a0, s3, 0x18
   11318: 0109d593     	srli	a1, s3, 0x10
   1131c: 0089d613     	srli	a2, s3, 0x8
   11320: 01895693     	srli	a3, s2, 0x18
   11324: 01095713     	srli	a4, s2, 0x10
   11328: 13310423     	sb	s3, 0x128(sp)
   1132c: 12c104a3     	sb	a2, 0x129(sp)
   11330: 12b10523     	sb	a1, 0x12a(sp)
   11334: 12a105a3     	sb	a0, 0x12b(sp)
   11338: 00895513     	srli	a0, s2, 0x8
   1133c: 13210223     	sb	s2, 0x124(sp)
   11340: 12a102a3     	sb	a0, 0x125(sp)
   11344: 12e10323     	sb	a4, 0x126(sp)
   11348: 12d103a3     	sb	a3, 0x127(sp)
   1134c: 07810513     	addi	a0, sp, 0x78
   11350: 0ec10593     	addi	a1, sp, 0xec
   11354: 0140006f     	j	0x11368 <_start+0x1f0>
   11358: 0d212c23     	sw	s2, 0xd8(sp)
   1135c: 0d312e23     	sw	s3, 0xdc(sp)
   11360: 07810513     	addi	a0, sp, 0x78
   11364: 00040593     	mv	a1, s0
   11368: 00000097     	auipc	ra, 0x0
   1136c: 178080e7     	jalr	0x178(ra) <sha2::sha256::compress256::h442137f2ea059990>
   11370: 000105b7     	lui	a1, 0x10
   11374: 07812503     	lw	a0, 0x78(sp)
   11378: 07c12603     	lw	a2, 0x7c(sp)
   1137c: 08012703     	lw	a4, 0x80(sp)
   11380: 08412783     	lw	a5, 0x84(sp)
   11384: 08812803     	lw	a6, 0x88(sp)
   11388: 08c12883     	lw	a7, 0x8c(sp)
   1138c: 09012283     	lw	t0, 0x90(sp)
   11390: 09412683     	lw	a3, 0x94(sp)
   11394: f0058593     	addi	a1, a1, -0x100
   11398: 00855313     	srli	t1, a0, 0x8
   1139c: 01855393     	srli	t2, a0, 0x18
   113a0: 00b57e33     	and	t3, a0, a1
   113a4: 01851513     	slli	a0, a0, 0x18
   113a8: 00865e93     	srli	t4, a2, 0x8
   113ac: 01865f13     	srli	t5, a2, 0x18
   113b0: 00b67fb3     	and	t6, a2, a1
   113b4: 01861613     	slli	a2, a2, 0x18
   113b8: 00875413     	srli	s0, a4, 0x8
   113bc: 01875493     	srli	s1, a4, 0x18
   113c0: 00b77933     	and	s2, a4, a1
   113c4: 01871713     	slli	a4, a4, 0x18
   113c8: 00b37333     	and	t1, t1, a1
   113cc: 00736333     	or	t1, t1, t2
   113d0: 0087d393     	srli	t2, a5, 0x8
   113d4: 008e1e13     	slli	t3, t3, 0x8
   113d8: 01c56533     	or	a0, a0, t3
   113dc: 0187de13     	srli	t3, a5, 0x18
   113e0: 00befeb3     	and	t4, t4, a1
   113e4: 01eeeeb3     	or	t4, t4, t5
   113e8: 00b7ff33     	and	t5, a5, a1
   113ec: 01879793     	slli	a5, a5, 0x18
   113f0: 008f9f93     	slli	t6, t6, 0x8
   113f4: 01f66633     	or	a2, a2, t6
   113f8: 00885f93     	srli	t6, a6, 0x8
   113fc: 00b47433     	and	s0, s0, a1
   11400: 00946433     	or	s0, s0, s1
   11404: 01885493     	srli	s1, a6, 0x18
   11408: 00891913     	slli	s2, s2, 0x8
   1140c: 01276733     	or	a4, a4, s2
   11410: 00b87933     	and	s2, a6, a1
   11414: 01881813     	slli	a6, a6, 0x18
   11418: 00b3f3b3     	and	t2, t2, a1
   1141c: 01c3e3b3     	or	t2, t2, t3
   11420: 0088de13     	srli	t3, a7, 0x8
   11424: 008f1f13     	slli	t5, t5, 0x8
   11428: 01e7e7b3     	or	a5, a5, t5
   1142c: 0188df13     	srli	t5, a7, 0x18
   11430: 00bfffb3     	and	t6, t6, a1
   11434: 009fefb3     	or	t6, t6, s1
   11438: 00b8f4b3     	and	s1, a7, a1
   1143c: 01889893     	slli	a7, a7, 0x18
   11440: 00891913     	slli	s2, s2, 0x8
   11444: 01286833     	or	a6, a6, s2
   11448: 0082d913     	srli	s2, t0, 0x8
   1144c: 00be7e33     	and	t3, t3, a1
   11450: 01ee6e33     	or	t3, t3, t5
   11454: 0182df13     	srli	t5, t0, 0x18
   11458: 00849493     	slli	s1, s1, 0x8
   1145c: 0098e8b3     	or	a7, a7, s1
   11460: 00b2f4b3     	and	s1, t0, a1
   11464: 01829293     	slli	t0, t0, 0x18
   11468: 00b97933     	and	s2, s2, a1
   1146c: 01e96f33     	or	t5, s2, t5
   11470: 0086d913     	srli	s2, a3, 0x8
   11474: 00849493     	slli	s1, s1, 0x8
   11478: 0092e2b3     	or	t0, t0, s1
   1147c: 0186d493     	srli	s1, a3, 0x18
   11480: 00b97933     	and	s2, s2, a1
   11484: 009964b3     	or	s1, s2, s1
   11488: 00b6f5b3     	and	a1, a3, a1
   1148c: 01869693     	slli	a3, a3, 0x18
   11490: 00859593     	slli	a1, a1, 0x8
   11494: 00b6e5b3     	or	a1, a3, a1
   11498: 00656533     	or	a0, a0, t1
   1149c: 01d66633     	or	a2, a2, t4
   114a0: 00876733     	or	a4, a4, s0
   114a4: 0077e6b3     	or	a3, a5, t2
   114a8: 01f867b3     	or	a5, a6, t6
   114ac: 01c8e833     	or	a6, a7, t3
   114b0: 01e2e8b3     	or	a7, t0, t5
   114b4: 0095e5b3     	or	a1, a1, s1
   114b8: 06a12c23     	sw	a0, 0x78(sp)
   114bc: 06c12e23     	sw	a2, 0x7c(sp)
   114c0: 08e12023     	sw	a4, 0x80(sp)
   114c4: 08d12223     	sw	a3, 0x84(sp)
   114c8: 08f12423     	sw	a5, 0x88(sp)
   114cc: 09012623     	sw	a6, 0x8c(sp)
   114d0: 09112823     	sw	a7, 0x90(sp)
   114d4: 08b12a23     	sw	a1, 0x94(sp)
   114d8: 07810513     	addi	a0, sp, 0x78
   114dc: 0000006f     	j	0x114dc <_start+0x364>

000114e0 <sha2::sha256::compress256::h442137f2ea059990>:
   114e0: ed010113     	addi	sp, sp, -0x130
   114e4: 12112623     	sw	ra, 0x12c(sp)
   114e8: 12812423     	sw	s0, 0x128(sp)
   114ec: 12912223     	sw	s1, 0x124(sp)
   114f0: 13212023     	sw	s2, 0x120(sp)
   114f4: 11312e23     	sw	s3, 0x11c(sp)
   114f8: 11412c23     	sw	s4, 0x118(sp)
   114fc: 11512a23     	sw	s5, 0x114(sp)
   11500: 11612823     	sw	s6, 0x110(sp)
   11504: 11712623     	sw	s7, 0x10c(sp)
   11508: 11812423     	sw	s8, 0x108(sp)
   1150c: 11912223     	sw	s9, 0x104(sp)
   11510: 11a12023     	sw	s10, 0x100(sp)
   11514: 0fb12e23     	sw	s11, 0xfc(sp)
   11518: 00050913     	mv	s2, a0
   1151c: 03c5c503     	lbu	a0, 0x3c(a1)
   11520: 03d5c603     	lbu	a2, 0x3d(a1)
   11524: 03e5c683     	lbu	a3, 0x3e(a1)
   11528: 0ed12423     	sw	a3, 0xe8(sp)
   1152c: 03f5ca83     	lbu	s5, 0x3f(a1)
   11530: 0385c803     	lbu	a6, 0x38(a1)
   11534: 0395c883     	lbu	a7, 0x39(a1)
   11538: 03a5c683     	lbu	a3, 0x3a(a1)
   1153c: 0ed12623     	sw	a3, 0xec(sp)
   11540: 03b5cb83     	lbu	s7, 0x3b(a1)
   11544: 0205c683     	lbu	a3, 0x20(a1)
   11548: 0215c783     	lbu	a5, 0x21(a1)
   1154c: 0225c703     	lbu	a4, 0x22(a1)
   11550: 0ee12023     	sw	a4, 0xe0(sp)
   11554: 0235c703     	lbu	a4, 0x23(a1)
   11558: 01c5c303     	lbu	t1, 0x1c(a1)
   1155c: 01d5c383     	lbu	t2, 0x1d(a1)
   11560: 01e5c283     	lbu	t0, 0x1e(a1)
   11564: 0e512223     	sw	t0, 0xe4(sp)
   11568: 01f5ca03     	lbu	s4, 0x1f(a1)
   1156c: 0045ce03     	lbu	t3, 0x4(a1)
   11570: 0055ce83     	lbu	t4, 0x5(a1)
   11574: 0065c283     	lbu	t0, 0x6(a1)
   11578: 0c512623     	sw	t0, 0xcc(sp)
   1157c: 0075cc83     	lbu	s9, 0x7(a1)
   11580: 0005c283     	lbu	t0, 0x0(a1)
   11584: 0015cf03     	lbu	t5, 0x1(a1)
   11588: 0025cf83     	lbu	t6, 0x2(a1)
   1158c: 0035c403     	lbu	s0, 0x3(a1)
   11590: 01061613     	slli	a2, a2, 0x10
   11594: 01851513     	slli	a0, a0, 0x18
   11598: 01089893     	slli	a7, a7, 0x10
   1159c: 01881813     	slli	a6, a6, 0x18
   115a0: 00c564b3     	or	s1, a0, a2
   115a4: 01186db3     	or	s11, a6, a7
   115a8: 0245c503     	lbu	a0, 0x24(a1)
   115ac: 0255c603     	lbu	a2, 0x25(a1)
   115b0: 0265c803     	lbu	a6, 0x26(a1)
   115b4: 0d012a23     	sw	a6, 0xd4(sp)
   115b8: 0275c803     	lbu	a6, 0x27(a1)
   115bc: 0d012823     	sw	a6, 0xd0(sp)
   115c0: 01079793     	slli	a5, a5, 0x10
   115c4: 01869693     	slli	a3, a3, 0x18
   115c8: 01039393     	slli	t2, t2, 0x10
   115cc: 01831313     	slli	t1, t1, 0x18
   115d0: 010e9e93     	slli	t4, t4, 0x10
   115d4: 018e1e13     	slli	t3, t3, 0x18
   115d8: 008f9f93     	slli	t6, t6, 0x8
   115dc: 00f6e0b3     	or	ra, a3, a5
   115e0: 007366b3     	or	a3, t1, t2
   115e4: 0cd12e23     	sw	a3, 0xdc(sp)
   115e8: 01de6b33     	or	s6, t3, t4
   115ec: 008fe6b3     	or	a3, t6, s0
   115f0: 0cd12423     	sw	a3, 0xc8(sp)
   115f4: 0185c683     	lbu	a3, 0x18(a1)
   115f8: 0195c783     	lbu	a5, 0x19(a1)
   115fc: 01a5c803     	lbu	a6, 0x1a(a1)
   11600: 0d012023     	sw	a6, 0xc0(sp)
   11604: 01b5cf83     	lbu	t6, 0x1b(a1)
   11608: 010f1f13     	slli	t5, t5, 0x10
   1160c: 01829293     	slli	t0, t0, 0x18
   11610: 01061613     	slli	a2, a2, 0x10
   11614: 01851513     	slli	a0, a0, 0x18
   11618: 01079793     	slli	a5, a5, 0x10
   1161c: 01869693     	slli	a3, a3, 0x18
   11620: 01e2e833     	or	a6, t0, t5
   11624: 0d012c23     	sw	a6, 0xd8(sp)
   11628: 00c56eb3     	or	t4, a0, a2
   1162c: 0145c503     	lbu	a0, 0x14(a1)
   11630: 0155c603     	lbu	a2, 0x15(a1)
   11634: 00f6ec33     	or	s8, a3, a5
   11638: 0165c683     	lbu	a3, 0x16(a1)
   1163c: 0cd12223     	sw	a3, 0xc4(sp)
   11640: 0175cf03     	lbu	t5, 0x17(a1)
   11644: 01061613     	slli	a2, a2, 0x10
   11648: 01851513     	slli	a0, a0, 0x18
   1164c: 00c56533     	or	a0, a0, a2
   11650: 0aa12823     	sw	a0, 0xb0(sp)
   11654: 0115c503     	lbu	a0, 0x11(a1)
   11658: 0105c603     	lbu	a2, 0x10(a1)
   1165c: 0125c683     	lbu	a3, 0x12(a1)
   11660: 0ad12e23     	sw	a3, 0xbc(sp)
   11664: 0135c683     	lbu	a3, 0x13(a1)
   11668: 01051513     	slli	a0, a0, 0x10
   1166c: 01861613     	slli	a2, a2, 0x18
   11670: 00a66d33     	or	s10, a2, a0
   11674: 00d5c503     	lbu	a0, 0xd(a1)
   11678: 00c5c603     	lbu	a2, 0xc(a1)
   1167c: 00e5c783     	lbu	a5, 0xe(a1)
   11680: 08f12823     	sw	a5, 0x90(sp)
   11684: 00f5c783     	lbu	a5, 0xf(a1)
   11688: 08f12423     	sw	a5, 0x88(sp)
   1168c: 01051513     	slli	a0, a0, 0x10
   11690: 01861613     	slli	a2, a2, 0x18
   11694: 00a66533     	or	a0, a2, a0
   11698: 08a12223     	sw	a0, 0x84(sp)
   1169c: 0315c503     	lbu	a0, 0x31(a1)
   116a0: 0305c603     	lbu	a2, 0x30(a1)
   116a4: 0325c783     	lbu	a5, 0x32(a1)
   116a8: 0af12623     	sw	a5, 0xac(sp)
   116ac: 0335c783     	lbu	a5, 0x33(a1)
   116b0: 08f12a23     	sw	a5, 0x94(sp)
   116b4: 01051513     	slli	a0, a0, 0x10
   116b8: 01861613     	slli	a2, a2, 0x18
   116bc: 00a66533     	or	a0, a2, a0
   116c0: 08a12623     	sw	a0, 0x8c(sp)
   116c4: 0095c503     	lbu	a0, 0x9(a1)
   116c8: 0085c603     	lbu	a2, 0x8(a1)
   116cc: 00a5c783     	lbu	a5, 0xa(a1)
   116d0: 0af12c23     	sw	a5, 0xb8(sp)
   116d4: 00b5c303     	lbu	t1, 0xb(a1)
   116d8: 01051513     	slli	a0, a0, 0x10
   116dc: 01861613     	slli	a2, a2, 0x18
   116e0: 00a66533     	or	a0, a2, a0
   116e4: 06a12a23     	sw	a0, 0x74(sp)
   116e8: 0295c503     	lbu	a0, 0x29(a1)
   116ec: 0285c603     	lbu	a2, 0x28(a1)
   116f0: 02a5c783     	lbu	a5, 0x2a(a1)
   116f4: 08f12023     	sw	a5, 0x80(sp)
   116f8: 02b5c783     	lbu	a5, 0x2b(a1)
   116fc: 06f12e23     	sw	a5, 0x7c(sp)
   11700: 01051513     	slli	a0, a0, 0x10
   11704: 01861613     	slli	a2, a2, 0x18
   11708: 00a66533     	or	a0, a2, a0
   1170c: 06a12823     	sw	a0, 0x70(sp)
   11710: 0355c503     	lbu	a0, 0x35(a1)
   11714: 0345c603     	lbu	a2, 0x34(a1)
   11718: 0365c783     	lbu	a5, 0x36(a1)
   1171c: 06f12623     	sw	a5, 0x6c(sp)
   11720: 0375c783     	lbu	a5, 0x37(a1)
   11724: 06f12423     	sw	a5, 0x68(sp)
   11728: 01051513     	slli	a0, a0, 0x10
   1172c: 01861613     	slli	a2, a2, 0x18
   11730: 00a66533     	or	a0, a2, a0
   11734: 06a12223     	sw	a0, 0x64(sp)
   11738: 02d5c503     	lbu	a0, 0x2d(a1)
   1173c: 02c5c603     	lbu	a2, 0x2c(a1)
   11740: 02e5c783     	lbu	a5, 0x2e(a1)
   11744: 04f12e23     	sw	a5, 0x5c(sp)
   11748: 02f5c583     	lbu	a1, 0x2f(a1)
   1174c: 04b12c23     	sw	a1, 0x58(sp)
   11750: 01051513     	slli	a0, a0, 0x10
   11754: 01861613     	slli	a2, a2, 0x18
   11758: 00a66533     	or	a0, a2, a0
   1175c: 04a12a23     	sw	a0, 0x54(sp)
   11760: 01092783     	lw	a5, 0x10(s2)
   11764: 0ef12823     	sw	a5, 0xf0(sp)
   11768: 01492503     	lw	a0, 0x14(s2)
   1176c: 0ea12c23     	sw	a0, 0xf8(sp)
   11770: 01892983     	lw	s3, 0x18(s2)
   11774: 01c92503     	lw	a0, 0x1c(s2)
   11778: 0aa12423     	sw	a0, 0xa8(sp)
   1177c: 0067d513     	srli	a0, a5, 0x6
   11780: 01a79593     	slli	a1, a5, 0x1a
   11784: 00a5e533     	or	a0, a1, a0
   11788: 00b7d593     	srli	a1, a5, 0xb
   1178c: 01579613     	slli	a2, a5, 0x15
   11790: 00b66633     	or	a2, a2, a1
   11794: 0197d593     	srli	a1, a5, 0x19
   11798: 00779793     	slli	a5, a5, 0x7
   1179c: 00b7e5b3     	or	a1, a5, a1
   117a0: 07212023     	sw	s2, 0x60(sp)
   117a4: 00092383     	lw	t2, 0x0(s2)
   117a8: 00492403     	lw	s0, 0x4(s2)
   117ac: 00892e03     	lw	t3, 0x8(s2)
   117b0: 00c92803     	lw	a6, 0xc(s2)
   117b4: 0b012223     	sw	a6, 0xa4(sp)
   117b8: 0e712a23     	sw	t2, 0xf4(sp)
   117bc: 0023d793     	srli	a5, t2, 0x2
   117c0: 01e39813     	slli	a6, t2, 0x1e
   117c4: 00f867b3     	or	a5, a6, a5
   117c8: 00d3d813     	srli	a6, t2, 0xd
   117cc: 01339893     	slli	a7, t2, 0x13
   117d0: 0108e833     	or	a6, a7, a6
   117d4: 0163d893     	srli	a7, t2, 0x16
   117d8: 00a39293     	slli	t0, t2, 0xa
   117dc: 0112e8b3     	or	a7, t0, a7
   117e0: 0d812283     	lw	t0, 0xd8(sp)
   117e4: 0c812903     	lw	s2, 0xc8(sp)
   117e8: 0122e2b3     	or	t0, t0, s2
   117ec: 04512823     	sw	t0, 0x50(sp)
   117f0: 00c54533     	xor	a0, a0, a2
   117f4: 0bc12023     	sw	t3, 0xa0(sp)
   117f8: 08812c23     	sw	s0, 0x98(sp)
   117fc: 008e4633     	xor	a2, t3, s0
   11800: 00767633     	and	a2, a2, t2
   11804: 008e72b3     	and	t0, t3, s0
   11808: 005642b3     	xor	t0, a2, t0
   1180c: 0107c633     	xor	a2, a5, a6
   11810: 00b54533     	xor	a0, a0, a1
   11814: 04a12623     	sw	a0, 0x4c(sp)
   11818: 011648b3     	xor	a7, a2, a7
   1181c: 0e812503     	lw	a0, 0xe8(sp)
   11820: 00851513     	slli	a0, a0, 0x8
   11824: 01556533     	or	a0, a0, s5
   11828: 019a9593     	slli	a1, s5, 0x19
   1182c: 00a4e633     	or	a2, s1, a0
   11830: 00765513     	srli	a0, a2, 0x7
   11834: 00a5e533     	or	a0, a1, a0
   11838: 04a12423     	sw	a0, 0x48(sp)
   1183c: 0124d513     	srli	a0, s1, 0x12
   11840: 00e61593     	slli	a1, a2, 0xe
   11844: 00a5e533     	or	a0, a1, a0
   11848: 04a12023     	sw	a0, 0x40(sp)
   1184c: 0e012503     	lw	a0, 0xe0(sp)
   11850: 00851513     	slli	a0, a0, 0x8
   11854: 00e56533     	or	a0, a0, a4
   11858: 01971713     	slli	a4, a4, 0x19
   1185c: 00a0e5b3     	or	a1, ra, a0
   11860: 0cb12423     	sw	a1, 0xc8(sp)
   11864: 0075d513     	srli	a0, a1, 0x7
   11868: 00a76533     	or	a0, a4, a0
   1186c: 02a12e23     	sw	a0, 0x3c(sp)
   11870: 0120d513     	srli	a0, ra, 0x12
   11874: 00e59593     	slli	a1, a1, 0xe
   11878: 00a5e533     	or	a0, a1, a0
   1187c: 02a12c23     	sw	a0, 0x38(sp)
   11880: 0cc12503     	lw	a0, 0xcc(sp)
   11884: 00851513     	slli	a0, a0, 0x8
   11888: 01956533     	or	a0, a0, s9
   1188c: 019c9593     	slli	a1, s9, 0x19
   11890: 00ab6733     	or	a4, s6, a0
   11894: 04e12223     	sw	a4, 0x44(sp)
   11898: 00775513     	srli	a0, a4, 0x7
   1189c: 00a5e533     	or	a0, a1, a0
   118a0: 0ca12623     	sw	a0, 0xcc(sp)
   118a4: 012b5513     	srli	a0, s6, 0x12
   118a8: 00e71593     	slli	a1, a4, 0xe
   118ac: 00a5e533     	or	a0, a1, a0
   118b0: 02a12a23     	sw	a0, 0x34(sp)
   118b4: 0ec12503     	lw	a0, 0xec(sp)
   118b8: 00851513     	slli	a0, a0, 0x8
   118bc: 01756533     	or	a0, a0, s7
   118c0: 00ade833     	or	a6, s11, a0
   118c4: 011dd513     	srli	a0, s11, 0x11
   118c8: 00f81593     	slli	a1, a6, 0xf
   118cc: 00a5e533     	or	a0, a1, a0
   118d0: 02a12823     	sw	a0, 0x30(sp)
   118d4: 013dd513     	srli	a0, s11, 0x13
   118d8: 00d81413     	slli	s0, a6, 0xd
   118dc: 00a46533     	or	a0, s0, a0
   118e0: 02a12423     	sw	a0, 0x28(sp)
   118e4: 0c012503     	lw	a0, 0xc0(sp)
   118e8: 00851513     	slli	a0, a0, 0x8
   118ec: 01f56533     	or	a0, a0, t6
   118f0: 019f9593     	slli	a1, t6, 0x19
   118f4: 00ac6733     	or	a4, s8, a0
   118f8: 0ee12023     	sw	a4, 0xe0(sp)
   118fc: 00775513     	srli	a0, a4, 0x7
   11900: 00a5e533     	or	a0, a1, a0
   11904: 02a12223     	sw	a0, 0x24(sp)
   11908: 012c5513     	srli	a0, s8, 0x12
   1190c: 00e71593     	slli	a1, a4, 0xe
   11910: 00a5e533     	or	a0, a1, a0
   11914: 02a12023     	sw	a0, 0x20(sp)
   11918: 0bc12503     	lw	a0, 0xbc(sp)
   1191c: 00851513     	slli	a0, a0, 0x8
   11920: 00d56533     	or	a0, a0, a3
   11924: 01969693     	slli	a3, a3, 0x19
   11928: 00ad65b3     	or	a1, s10, a0
   1192c: 0ab12a23     	sw	a1, 0xb4(sp)
   11930: 0075d513     	srli	a0, a1, 0x7
   11934: 00a6e533     	or	a0, a3, a0
   11938: 00a12e23     	sw	a0, 0x1c(sp)
   1193c: 012d5513     	srli	a0, s10, 0x12
   11940: 00e59593     	slli	a1, a1, 0xe
   11944: 00a5e533     	or	a0, a1, a0
   11948: 00a12c23     	sw	a0, 0x18(sp)
   1194c: 0b812503     	lw	a0, 0xb8(sp)
   11950: 00851513     	slli	a0, a0, 0x8
   11954: 00656533     	or	a0, a0, t1
   11958: 01931313     	slli	t1, t1, 0x19
   1195c: 07412583     	lw	a1, 0x74(sp)
   11960: 00a5e6b3     	or	a3, a1, a0
   11964: 02d12623     	sw	a3, 0x2c(sp)
   11968: 0076d513     	srli	a0, a3, 0x7
   1196c: 00a36533     	or	a0, t1, a0
   11970: 00a12a23     	sw	a0, 0x14(sp)
   11974: 0125d513     	srli	a0, a1, 0x12
   11978: 00e69593     	slli	a1, a3, 0xe
   1197c: 00a5e533     	or	a0, a1, a0
   11980: 00a12823     	sw	a0, 0x10(sp)
   11984: 0114d513     	srli	a0, s1, 0x11
   11988: 0cc12c23     	sw	a2, 0xd8(sp)
   1198c: 00f61593     	slli	a1, a2, 0xf
   11990: 00a5e533     	or	a0, a1, a0
   11994: 00a12623     	sw	a0, 0xc(sp)
   11998: 0134d493     	srli	s1, s1, 0x13
   1199c: 00d61513     	slli	a0, a2, 0xd
   119a0: 00956533     	or	a0, a0, s1
   119a4: 00a12423     	sw	a0, 0x8(sp)
   119a8: 06c12503     	lw	a0, 0x6c(sp)
   119ac: 00851513     	slli	a0, a0, 0x8
   119b0: 06812583     	lw	a1, 0x68(sp)
   119b4: 00b56533     	or	a0, a0, a1
   119b8: 01959593     	slli	a1, a1, 0x19
   119bc: 06412683     	lw	a3, 0x64(sp)
   119c0: 00a6e633     	or	a2, a3, a0
   119c4: 0cc12023     	sw	a2, 0xc0(sp)
   119c8: 00765513     	srli	a0, a2, 0x7
   119cc: 00a5e533     	or	a0, a1, a0
   119d0: 06a12623     	sw	a0, 0x6c(sp)
   119d4: 0126d513     	srli	a0, a3, 0x12
   119d8: 00e61593     	slli	a1, a2, 0xe
   119dc: 00a5ed33     	or	s10, a1, a0
   119e0: 05c12503     	lw	a0, 0x5c(sp)
   119e4: 00851513     	slli	a0, a0, 0x8
   119e8: 05812583     	lw	a1, 0x58(sp)
   119ec: 00b56533     	or	a0, a0, a1
   119f0: 01959593     	slli	a1, a1, 0x19
   119f4: 05412683     	lw	a3, 0x54(sp)
   119f8: 00a6e733     	or	a4, a3, a0
   119fc: 0ae12c23     	sw	a4, 0xb8(sp)
   11a00: 00775793     	srli	a5, a4, 0x7
   11a04: 00f5e5b3     	or	a1, a1, a5
   11a08: 06b12423     	sw	a1, 0x68(sp)
   11a0c: 0126d513     	srli	a0, a3, 0x12
   11a10: 00e71713     	slli	a4, a4, 0xe
   11a14: 00a76ab3     	or	s5, a4, a0
   11a18: 0d412503     	lw	a0, 0xd4(sp)
   11a1c: 00851513     	slli	a0, a0, 0x8
   11a20: 0d012c83     	lw	s9, 0xd0(sp)
   11a24: 01956533     	or	a0, a0, s9
   11a28: 019c9593     	slli	a1, s9, 0x19
   11a2c: 00aee633     	or	a2, t4, a0
   11a30: 0ac12e23     	sw	a2, 0xbc(sp)
   11a34: 00765693     	srli	a3, a2, 0x7
   11a38: 00d5e5b3     	or	a1, a1, a3
   11a3c: 0cb12a23     	sw	a1, 0xd4(sp)
   11a40: 012ed513     	srli	a0, t4, 0x12
   11a44: 00e61613     	slli	a2, a2, 0xe
   11a48: 00a66fb3     	or	t6, a2, a0
   11a4c: 0e412503     	lw	a0, 0xe4(sp)
   11a50: 00851513     	slli	a0, a0, 0x8
   11a54: 01456533     	or	a0, a0, s4
   11a58: 019a1593     	slli	a1, s4, 0x19
   11a5c: 0dc12c83     	lw	s9, 0xdc(sp)
   11a60: 00aceb33     	or	s6, s9, a0
   11a64: 007b5513     	srli	a0, s6, 0x7
   11a68: 00a5e433     	or	s0, a1, a0
   11a6c: 012cd513     	srli	a0, s9, 0x12
   11a70: 00eb1493     	slli	s1, s6, 0xe
   11a74: 0f612423     	sw	s6, 0xe8(sp)
   11a78: 00a4eeb3     	or	t4, s1, a0
   11a7c: 0c412483     	lw	s1, 0xc4(sp)
   11a80: 00849493     	slli	s1, s1, 0x8
   11a84: 01e4e4b3     	or	s1, s1, t5
   11a88: 019f1513     	slli	a0, t5, 0x19
   11a8c: 0b012583     	lw	a1, 0xb0(sp)
   11a90: 0095ea33     	or	s4, a1, s1
   11a94: 0d412823     	sw	s4, 0xd0(sp)
   11a98: 007a5493     	srli	s1, s4, 0x7
   11a9c: 00956f33     	or	t5, a0, s1
   11aa0: 0125d493     	srli	s1, a1, 0x12
   11aa4: 00ea1a13     	slli	s4, s4, 0xe
   11aa8: 009a6e33     	or	t3, s4, s1
   11aac: 09012483     	lw	s1, 0x90(sp)
   11ab0: 00849493     	slli	s1, s1, 0x8
   11ab4: 08812503     	lw	a0, 0x88(sp)
   11ab8: 00a4e4b3     	or	s1, s1, a0
   11abc: 01951513     	slli	a0, a0, 0x19
   11ac0: 08412583     	lw	a1, 0x84(sp)
   11ac4: 0095e933     	or	s2, a1, s1
   11ac8: 0f212623     	sw	s2, 0xec(sp)
   11acc: 00795493     	srli	s1, s2, 0x7
   11ad0: 009563b3     	or	t2, a0, s1
   11ad4: 0125d493     	srli	s1, a1, 0x12
   11ad8: 00e91913     	slli	s2, s2, 0xe
   11adc: 009964b3     	or	s1, s2, s1
   11ae0: 019b9913     	slli	s2, s7, 0x19
   11ae4: 00080a13     	mv	s4, a6
   11ae8: 00785b93     	srli	s7, a6, 0x7
   11aec: 01796933     	or	s2, s2, s7
   11af0: 012ddb93     	srli	s7, s11, 0x12
   11af4: 00e81d93     	slli	s11, a6, 0xe
   11af8: 017de7b3     	or	a5, s11, s7
   11afc: 0ac12b83     	lw	s7, 0xac(sp)
   11b00: 008b9b93     	slli	s7, s7, 0x8
   11b04: 09412503     	lw	a0, 0x94(sp)
   11b08: 00abebb3     	or	s7, s7, a0
   11b0c: 01951513     	slli	a0, a0, 0x19
   11b10: 08c12603     	lw	a2, 0x8c(sp)
   11b14: 01766cb3     	or	s9, a2, s7
   11b18: 0b912623     	sw	s9, 0xac(sp)
   11b1c: 007cdb93     	srli	s7, s9, 0x7
   11b20: 01756333     	or	t1, a0, s7
   11b24: 01265b93     	srli	s7, a2, 0x12
   11b28: 00ec9c93     	slli	s9, s9, 0xe
   11b2c: 017cecb3     	or	s9, s9, s7
   11b30: 08012b83     	lw	s7, 0x80(sp)
   11b34: 008b9b93     	slli	s7, s7, 0x8
   11b38: 07c12503     	lw	a0, 0x7c(sp)
   11b3c: 00abebb3     	or	s7, s7, a0
   11b40: 01951513     	slli	a0, a0, 0x19
   11b44: 07012583     	lw	a1, 0x70(sp)
   11b48: 0175e0b3     	or	ra, a1, s7
   11b4c: 0070db93     	srli	s7, ra, 0x7
   11b50: 01756733     	or	a4, a0, s7
   11b54: 0125db93     	srli	s7, a1, 0x12
   11b58: 00e09c13     	slli	s8, ra, 0xe
   11b5c: 017c6c33     	or	s8, s8, s7
   11b60: 09312e23     	sw	s3, 0x9c(sp)
   11b64: 0f812503     	lw	a0, 0xf8(sp)
   11b68: 00a9cbb3     	xor	s7, s3, a0
   11b6c: 0f012503     	lw	a0, 0xf0(sp)
   11b70: 00abfbb3     	and	s7, s7, a0
   11b74: 013bcbb3     	xor	s7, s7, s3
   11b78: 0a812503     	lw	a0, 0xa8(sp)
   11b7c: 01750bb3     	add	s7, a0, s7
   11b80: 04c12503     	lw	a0, 0x4c(sp)
   11b84: 00ab8bb3     	add	s7, s7, a0
   11b88: 011288b3     	add	a7, t0, a7
   11b8c: 0d112e23     	sw	a7, 0xdc(sp)
   11b90: 04812503     	lw	a0, 0x48(sp)
   11b94: 04012583     	lw	a1, 0x40(sp)
   11b98: 00b54533     	xor	a0, a0, a1
   11b9c: 03c12583     	lw	a1, 0x3c(sp)
   11ba0: 03812603     	lw	a2, 0x38(sp)
   11ba4: 00c5c5b3     	xor	a1, a1, a2
   11ba8: 0cc12603     	lw	a2, 0xcc(sp)
   11bac: 03412683     	lw	a3, 0x34(sp)
   11bb0: 00d64633     	xor	a2, a2, a3
   11bb4: 03012683     	lw	a3, 0x30(sp)
   11bb8: 02812803     	lw	a6, 0x28(sp)
   11bbc: 0106c6b3     	xor	a3, a3, a6
   11bc0: 02412803     	lw	a6, 0x24(sp)
   11bc4: 02012883     	lw	a7, 0x20(sp)
   11bc8: 01184db3     	xor	s11, a6, a7
   11bcc: 01c12803     	lw	a6, 0x1c(sp)
   11bd0: 01812883     	lw	a7, 0x18(sp)
   11bd4: 01184833     	xor	a6, a6, a7
   11bd8: 01412883     	lw	a7, 0x14(sp)
   11bdc: 01012283     	lw	t0, 0x10(sp)
   11be0: 0058c8b3     	xor	a7, a7, t0
   11be4: 00c12283     	lw	t0, 0xc(sp)
   11be8: 00812983     	lw	s3, 0x8(sp)
   11bec: 0132c2b3     	xor	t0, t0, s3
   11bf0: 06c12983     	lw	s3, 0x6c(sp)
   11bf4: 01a9cd33     	xor	s10, s3, s10
   11bf8: 06812983     	lw	s3, 0x68(sp)
   11bfc: 0159cab3     	xor	s5, s3, s5
   11c00: 0d412983     	lw	s3, 0xd4(sp)
   11c04: 01f9cfb3     	xor	t6, s3, t6
   11c08: 01d44eb3     	xor	t4, s0, t4
   11c0c: 01cf4e33     	xor	t3, t5, t3
   11c10: 0093c3b3     	xor	t2, t2, s1
   11c14: 00f949b3     	xor	s3, s2, a5
   11c18: 01934333     	xor	t1, t1, s9
   11c1c: 01874733     	xor	a4, a4, s8
   11c20: 0ee12223     	sw	a4, 0xe4(sp)
   11c24: 428a3437     	lui	s0, 0x428a3
   11c28: f9840413     	addi	s0, s0, -0x68
   11c2c: 05012783     	lw	a5, 0x50(sp)
   11c30: 00878433     	add	s0, a5, s0
   11c34: 008b8433     	add	s0, s7, s0
   11c38: 0d812903     	lw	s2, 0xd8(sp)
   11c3c: 00395493     	srli	s1, s2, 0x3
   11c40: 00954533     	xor	a0, a0, s1
   11c44: 0aa12823     	sw	a0, 0xb0(sp)
   11c48: 0c812503     	lw	a0, 0xc8(sp)
   11c4c: 00355513     	srli	a0, a0, 0x3
   11c50: 00a5c533     	xor	a0, a1, a0
   11c54: 08a12a23     	sw	a0, 0x94(sp)
   11c58: 04412703     	lw	a4, 0x44(sp)
   11c5c: 00375513     	srli	a0, a4, 0x3
   11c60: 00a64f33     	xor	t5, a2, a0
   11c64: 07412c23     	sw	s4, 0x78(sp)
   11c68: 00aa5593     	srli	a1, s4, 0xa
   11c6c: 00b6c4b3     	xor	s1, a3, a1
   11c70: 0e012c83     	lw	s9, 0xe0(sp)
   11c74: 003cd593     	srli	a1, s9, 0x3
   11c78: 00bdc5b3     	xor	a1, s11, a1
   11c7c: 0b412d83     	lw	s11, 0xb4(sp)
   11c80: 003dd613     	srli	a2, s11, 0x3
   11c84: 00c84533     	xor	a0, a6, a2
   11c88: 02c12c03     	lw	s8, 0x2c(sp)
   11c8c: 003c5613     	srli	a2, s8, 0x3
   11c90: 00c8c833     	xor	a6, a7, a2
   11c94: 00a95613     	srli	a2, s2, 0xa
   11c98: 00c2c8b3     	xor	a7, t0, a2
   11c9c: 0c012683     	lw	a3, 0xc0(sp)
   11ca0: 0036d613     	srli	a2, a3, 0x3
   11ca4: 00cd4633     	xor	a2, s10, a2
   11ca8: 08c12223     	sw	a2, 0x84(sp)
   11cac: 0b812b83     	lw	s7, 0xb8(sp)
   11cb0: 003bd613     	srli	a2, s7, 0x3
   11cb4: 00cac633     	xor	a2, s5, a2
   11cb8: 08c12623     	sw	a2, 0x8c(sp)
   11cbc: 0bc12a83     	lw	s5, 0xbc(sp)
   11cc0: 003ad613     	srli	a2, s5, 0x3
   11cc4: 00cfc633     	xor	a2, t6, a2
   11cc8: 08c12823     	sw	a2, 0x90(sp)
   11ccc: 003b5613     	srli	a2, s6, 0x3
   11cd0: 00cec2b3     	xor	t0, t4, a2
   11cd4: 0d012e83     	lw	t4, 0xd0(sp)
   11cd8: 003ed613     	srli	a2, t4, 0x3
   11cdc: 00ce4e33     	xor	t3, t3, a2
   11ce0: 0ec12f83     	lw	t6, 0xec(sp)
   11ce4: 003fd613     	srli	a2, t6, 0x3
   11ce8: 00c3c3b3     	xor	t2, t2, a2
   11cec: 003a5613     	srli	a2, s4, 0x3
   11cf0: 00c9c633     	xor	a2, s3, a2
   11cf4: 06c12423     	sw	a2, 0x68(sp)
   11cf8: 0ac12b03     	lw	s6, 0xac(sp)
   11cfc: 003b5613     	srli	a2, s6, 0x3
   11d00: 00c34633     	xor	a2, t1, a2
   11d04: 04c12423     	sw	a2, 0x48(sp)
   11d08: 06112a23     	sw	ra, 0x74(sp)
   11d0c: 0030d613     	srli	a2, ra, 0x3
   11d10: 0e412303     	lw	t1, 0xe4(sp)
   11d14: 00c34633     	xor	a2, t1, a2
   11d18: 06c12623     	sw	a2, 0x6c(sp)
   11d1c: 01578633     	add	a2, a5, s5
   11d20: 00cf0f33     	add	t5, t5, a2
   11d24: 014e8633     	add	a2, t4, s4
   11d28: 00c585b3     	add	a1, a1, a2
   11d2c: 0cb12a23     	sw	a1, 0xd4(sp)
   11d30: 016f85b3     	add	a1, t6, s6
   11d34: 00b505b3     	add	a1, a0, a1
   11d38: 00170533     	add	a0, a4, ra
   11d3c: 00070093     	mv	ra, a4
   11d40: 00a80333     	add	t1, a6, a0
   11d44: 012c8833     	add	a6, s9, s2
   11d48: 01028833     	add	a6, t0, a6
   11d4c: 07012823     	sw	a6, 0x70(sp)
   11d50: 00dd8533     	add	a0, s11, a3
   11d54: 00ae0533     	add	a0, t3, a0
   11d58: 08a12423     	sw	a0, 0x88(sp)
   11d5c: 017c07b3     	add	a5, s8, s7
   11d60: 00f387b3     	add	a5, t2, a5
   11d64: 009f06b3     	add	a3, t5, s1
   11d68: 01130533     	add	a0, t1, a7
   11d6c: 0dc12e03     	lw	t3, 0xdc(sp)
   11d70: 008e0e33     	add	t3, t3, s0
   11d74: 0a412983     	lw	s3, 0xa4(sp)
   11d78: 013409b3     	add	s3, s0, s3
   11d7c: 0069d713     	srli	a4, s3, 0x6
   11d80: 01a99893     	slli	a7, s3, 0x1a
   11d84: 00e8e8b3     	or	a7, a7, a4
   11d88: 00b9d713     	srli	a4, s3, 0xb
   11d8c: 01599293     	slli	t0, s3, 0x15
   11d90: 00e2e2b3     	or	t0, t0, a4
   11d94: 0199d713     	srli	a4, s3, 0x19
   11d98: 00799313     	slli	t1, s3, 0x7
   11d9c: 00e36733     	or	a4, t1, a4
   11da0: 002e5313     	srli	t1, t3, 0x2
   11da4: 01ee1393     	slli	t2, t3, 0x1e
   11da8: 0063e333     	or	t1, t2, t1
   11dac: 00de5393     	srli	t2, t3, 0xd
   11db0: 013e1e93     	slli	t4, t3, 0x13
   11db4: 007ee3b3     	or	t2, t4, t2
   11db8: 016e5e93     	srli	t4, t3, 0x16
   11dbc: 00ae1f13     	slli	t5, t3, 0xa
   11dc0: 01df6eb3     	or	t4, t5, t4
   11dc4: 09812c83     	lw	s9, 0x98(sp)
   11dc8: 0f412803     	lw	a6, 0xf4(sp)
   11dcc: 010ccf33     	xor	t5, s9, a6
   11dd0: 01ee7f33     	and	t5, t3, t5
   11dd4: 010cffb3     	and	t6, s9, a6
   11dd8: 01ff4f33     	xor	t5, t5, t6
   11ddc: 09c12603     	lw	a2, 0x9c(sp)
   11de0: 001600b3     	add	ra, a2, ra
   11de4: 0f812903     	lw	s2, 0xf8(sp)
   11de8: 0f012a03     	lw	s4, 0xf0(sp)
   11dec: 01494fb3     	xor	t6, s2, s4
   11df0: 01f9ffb3     	and	t6, s3, t6
   11df4: 012fcfb3     	xor	t6, t6, s2
   11df8: 01f08fb3     	add	t6, ra, t6
   11dfc: 0058c8b3     	xor	a7, a7, t0
   11e00: 007342b3     	xor	t0, t1, t2
   11e04: 01155313     	srli	t1, a0, 0x11
   11e08: 00f51393     	slli	t2, a0, 0xf
   11e0c: 0063e333     	or	t1, t2, t1
   11e10: 01355393     	srli	t2, a0, 0x13
   11e14: 00d51413     	slli	s0, a0, 0xd
   11e18: 007463b3     	or	t2, s0, t2
   11e1c: 0116d413     	srli	s0, a3, 0x11
   11e20: 00f69493     	slli	s1, a3, 0xf
   11e24: 0084e433     	or	s0, s1, s0
   11e28: 0136d493     	srli	s1, a3, 0x13
   11e2c: 00d69a93     	slli	s5, a3, 0xd
   11e30: 009ae4b3     	or	s1, s5, s1
   11e34: 0076da93     	srli	s5, a3, 0x7
   11e38: 01969b13     	slli	s6, a3, 0x19
   11e3c: 015b6ab3     	or	s5, s6, s5
   11e40: 0126db13     	srli	s6, a3, 0x12
   11e44: 00e69b93     	slli	s7, a3, 0xe
   11e48: 016beb33     	or	s6, s7, s6
   11e4c: 00755b93     	srli	s7, a0, 0x7
   11e50: 01951d13     	slli	s10, a0, 0x19
   11e54: 017d6bb3     	or	s7, s10, s7
   11e58: 01255d13     	srli	s10, a0, 0x12
   11e5c: 00e51d93     	slli	s11, a0, 0xe
   11e60: 01aded33     	or	s10, s11, s10
   11e64: 00e8c733     	xor	a4, a7, a4
   11e68: 713748b7     	lui	a7, 0x71374
   11e6c: 49188893     	addi	a7, a7, 0x491
   11e70: 011f88b3     	add	a7, t6, a7
   11e74: 01d2c2b3     	xor	t0, t0, t4
   11e78: 00734333     	xor	t1, t1, t2
   11e7c: 00944433     	xor	s0, s0, s1
   11e80: 016ac3b3     	xor	t2, s5, s6
   11e84: 01abceb3     	xor	t4, s7, s10
   11e88: 00e888b3     	add	a7, a7, a4
   11e8c: 01e282b3     	add	t0, t0, t5
   11e90: 0ea12223     	sw	a0, 0xe4(sp)
   11e94: 00a55713     	srli	a4, a0, 0xa
   11e98: 00e34333     	xor	t1, t1, a4
   11e9c: 0cd12623     	sw	a3, 0xcc(sp)
   11ea0: 00a6d713     	srli	a4, a3, 0xa
   11ea4: 00e44433     	xor	s0, s0, a4
   11ea8: 0036d713     	srli	a4, a3, 0x3
   11eac: 00e3c633     	xor	a2, t2, a4
   11eb0: 04c12823     	sw	a2, 0x50(sp)
   11eb4: 00355393     	srli	t2, a0, 0x3
   11eb8: 007ec533     	xor	a0, t4, t2
   11ebc: 06a12e23     	sw	a0, 0x7c(sp)
   11ec0: 006586b3     	add	a3, a1, t1
   11ec4: 00878633     	add	a2, a5, s0
   11ec8: 01128f33     	add	t5, t0, a7
   11ecc: 0a012583     	lw	a1, 0xa0(sp)
   11ed0: 00b885b3     	add	a1, a7, a1
   11ed4: 0065d793     	srli	a5, a1, 0x6
   11ed8: 01a59893     	slli	a7, a1, 0x1a
   11edc: 00f8e7b3     	or	a5, a7, a5
   11ee0: 00b5d893     	srli	a7, a1, 0xb
   11ee4: 01559293     	slli	t0, a1, 0x15
   11ee8: 0112e2b3     	or	t0, t0, a7
   11eec: 0195d893     	srli	a7, a1, 0x19
   11ef0: 00759313     	slli	t1, a1, 0x7
   11ef4: 011368b3     	or	a7, t1, a7
   11ef8: 002f5313     	srli	t1, t5, 0x2
   11efc: 01ef1393     	slli	t2, t5, 0x1e
   11f00: 0063e333     	or	t1, t2, t1
   11f04: 00df5393     	srli	t2, t5, 0xd
   11f08: 013f1e93     	slli	t4, t5, 0x13
   11f0c: 007ee3b3     	or	t2, t4, t2
   11f10: 016f5e93     	srli	t4, t5, 0x16
   11f14: 00af1f93     	slli	t6, t5, 0xa
   11f18: 01dfeeb3     	or	t4, t6, t4
   11f1c: 010e4fb3     	xor	t6, t3, a6
   11f20: 01ff7fb3     	and	t6, t5, t6
   11f24: 010e7433     	and	s0, t3, a6
   11f28: 008fcfb3     	xor	t6, t6, s0
   11f2c: 0116d413     	srli	s0, a3, 0x11
   11f30: 00f69493     	slli	s1, a3, 0xf
   11f34: 0084e433     	or	s0, s1, s0
   11f38: 0136d493     	srli	s1, a3, 0x13
   11f3c: 00d69a93     	slli	s5, a3, 0xd
   11f40: 009ae4b3     	or	s1, s5, s1
   11f44: 01165a93     	srli	s5, a2, 0x11
   11f48: 00f61b13     	slli	s6, a2, 0xf
   11f4c: 015b6ab3     	or	s5, s6, s5
   11f50: 01365b13     	srli	s6, a2, 0x13
   11f54: 00d61b93     	slli	s7, a2, 0xd
   11f58: 016beb33     	or	s6, s7, s6
   11f5c: 0076db93     	srli	s7, a3, 0x7
   11f60: 01969d13     	slli	s10, a3, 0x19
   11f64: 017d6bb3     	or	s7, s10, s7
   11f68: 0126dd13     	srli	s10, a3, 0x12
   11f6c: 00e69d93     	slli	s11, a3, 0xe
   11f70: 01aded33     	or	s10, s11, s10
   11f74: 00765d93     	srli	s11, a2, 0x7
   11f78: 01961093     	slli	ra, a2, 0x19
   11f7c: 01b0edb3     	or	s11, ra, s11
   11f80: 01265093     	srli	ra, a2, 0x12
   11f84: 00e61713     	slli	a4, a2, 0xe
   11f88: 00176733     	or	a4, a4, ra
   11f8c: 018900b3     	add	ra, s2, s8
   11f90: 0149c933     	xor	s2, s3, s4
   11f94: 0125f933     	and	s2, a1, s2
   11f98: 01494933     	xor	s2, s2, s4
   11f9c: 01208933     	add	s2, ra, s2
   11fa0: 0057c7b3     	xor	a5, a5, t0
   11fa4: 007342b3     	xor	t0, t1, t2
   11fa8: 00944433     	xor	s0, s0, s1
   11fac: 016ac333     	xor	t1, s5, s6
   11fb0: 01abc3b3     	xor	t2, s7, s10
   11fb4: 00edc733     	xor	a4, s11, a4
   11fb8: 0117c7b3     	xor	a5, a5, a7
   11fbc: b5c108b7     	lui	a7, 0xb5c10
   11fc0: bcf88893     	addi	a7, a7, -0x431
   11fc4: 011908b3     	add	a7, s2, a7
   11fc8: 01d2c2b3     	xor	t0, t0, t4
   11fcc: 0cd12223     	sw	a3, 0xc4(sp)
   11fd0: 00a6de93     	srli	t4, a3, 0xa
   11fd4: 01d44eb3     	xor	t4, s0, t4
   11fd8: 0cc12e23     	sw	a2, 0xdc(sp)
   11fdc: 00a65413     	srli	s0, a2, 0xa
   11fe0: 00834433     	xor	s0, t1, s0
   11fe4: 0036d313     	srli	t1, a3, 0x3
   11fe8: 0063c533     	xor	a0, t2, t1
   11fec: 08a12023     	sw	a0, 0x80(sp)
   11ff0: 00365313     	srli	t1, a2, 0x3
   11ff4: 00674533     	xor	a0, a4, t1
   11ff8: 04a12a23     	sw	a0, 0x54(sp)
   11ffc: 00f88733     	add	a4, a7, a5
   12000: 01f280b3     	add	ra, t0, t6
   12004: 0d412c03     	lw	s8, 0xd4(sp)
   12008: 01dc0c33     	add	s8, s8, t4
   1200c: 08812503     	lw	a0, 0x88(sp)
   12010: 00850833     	add	a6, a0, s0
   12014: 011c5513     	srli	a0, s8, 0x11
   12018: 00fc1613     	slli	a2, s8, 0xf
   1201c: 00a66633     	or	a2, a2, a0
   12020: 013c5513     	srli	a0, s8, 0x13
   12024: 00dc1793     	slli	a5, s8, 0xd
   12028: 00a7e7b3     	or	a5, a5, a0
   1202c: 01185513     	srli	a0, a6, 0x11
   12030: 00f81893     	slli	a7, a6, 0xf
   12034: 00a8e8b3     	or	a7, a7, a0
   12038: 01385513     	srli	a0, a6, 0x13
   1203c: 00d81293     	slli	t0, a6, 0xd
   12040: 00a2e2b3     	or	t0, t0, a0
   12044: 007c5513     	srli	a0, s8, 0x7
   12048: 019c1393     	slli	t2, s8, 0x19
   1204c: 00a3eeb3     	or	t4, t2, a0
   12050: 012c5513     	srli	a0, s8, 0x12
   12054: 00ec1393     	slli	t2, s8, 0xe
   12058: 00a3e3b3     	or	t2, t2, a0
   1205c: 00785513     	srli	a0, a6, 0x7
   12060: 01981413     	slli	s0, a6, 0x19
   12064: 00a46433     	or	s0, s0, a0
   12068: 01285513     	srli	a0, a6, 0x12
   1206c: 00e81f93     	slli	t6, a6, 0xe
   12070: 00afe4b3     	or	s1, t6, a0
   12074: 0b412f83     	lw	t6, 0xb4(sp)
   12078: 01f98fb3     	add	t6, s3, t6
   1207c: 00e080b3     	add	ra, ra, a4
   12080: 01970533     	add	a0, a4, s9
   12084: 0135c733     	xor	a4, a1, s3
   12088: 00e57733     	and	a4, a0, a4
   1208c: 01374733     	xor	a4, a4, s3
   12090: 00655913     	srli	s2, a0, 0x6
   12094: 01a51993     	slli	s3, a0, 0x1a
   12098: 0129e933     	or	s2, s3, s2
   1209c: 00b55993     	srli	s3, a0, 0xb
   120a0: 01551a93     	slli	s5, a0, 0x15
   120a4: 013ae9b3     	or	s3, s5, s3
   120a8: 01955a93     	srli	s5, a0, 0x19
   120ac: 00751b13     	slli	s6, a0, 0x7
   120b0: 015b6ab3     	or	s5, s6, s5
   120b4: 0020db13     	srli	s6, ra, 0x2
   120b8: 01e09b93     	slli	s7, ra, 0x1e
   120bc: 016beb33     	or	s6, s7, s6
   120c0: 00d0db93     	srli	s7, ra, 0xd
   120c4: 01309d13     	slli	s10, ra, 0x13
   120c8: 017d6bb3     	or	s7, s10, s7
   120cc: 0160dd13     	srli	s10, ra, 0x16
   120d0: 00a09d93     	slli	s11, ra, 0xa
   120d4: 01aded33     	or	s10, s11, s10
   120d8: 01cf4db3     	xor	s11, t5, t3
   120dc: 01b0fdb3     	and	s11, ra, s11
   120e0: 01cf7333     	and	t1, t5, t3
   120e4: 006dc333     	xor	t1, s11, t1
   120e8: 00f64633     	xor	a2, a2, a5
   120ec: 0058c7b3     	xor	a5, a7, t0
   120f0: 007ec8b3     	xor	a7, t4, t2
   120f4: 00944433     	xor	s0, s0, s1
   120f8: 0ec12283     	lw	t0, 0xec(sp)
   120fc: 005a02b3     	add	t0, s4, t0
   12100: 00e28733     	add	a4, t0, a4
   12104: 013943b3     	xor	t2, s2, s3
   12108: 017b4eb3     	xor	t4, s6, s7
   1210c: 0d812a23     	sw	s8, 0xd4(sp)
   12110: 00ac5293     	srli	t0, s8, 0xa
   12114: 00564633     	xor	a2, a2, t0
   12118: 00080993     	mv	s3, a6
   1211c: 00a85293     	srli	t0, a6, 0xa
   12120: 0057c7b3     	xor	a5, a5, t0
   12124: 003c5293     	srli	t0, s8, 0x3
   12128: 0058c6b3     	xor	a3, a7, t0
   1212c: 06d12223     	sw	a3, 0x64(sp)
   12130: 00385893     	srli	a7, a6, 0x3
   12134: 09012423     	sw	a6, 0x88(sp)
   12138: 011446b3     	xor	a3, s0, a7
   1213c: 04d12e23     	sw	a3, 0x5c(sp)
   12140: 0153c8b3     	xor	a7, t2, s5
   12144: e9b5e3b7     	lui	t2, 0xe9b5e
   12148: ba538393     	addi	t2, t2, -0x45b
   1214c: 00770733     	add	a4, a4, t2
   12150: 01aec3b3     	xor	t2, t4, s10
   12154: 0e812c03     	lw	s8, 0xe8(sp)
   12158: 09412683     	lw	a3, 0x94(sp)
   1215c: 01868c33     	add	s8, a3, s8
   12160: 0cc12683     	lw	a3, 0xcc(sp)
   12164: 00dc0c33     	add	s8, s8, a3
   12168: 00cc06b3     	add	a3, s8, a2
   1216c: 07012c83     	lw	s9, 0x70(sp)
   12170: 00fc8d33     	add	s10, s9, a5
   12174: 01170633     	add	a2, a4, a7
   12178: 00638333     	add	t1, t2, t1
   1217c: 011d5713     	srli	a4, s10, 0x11
   12180: 00fd1793     	slli	a5, s10, 0xf
   12184: 00e7e733     	or	a4, a5, a4
   12188: 013d5793     	srli	a5, s10, 0x13
   1218c: 00dd1813     	slli	a6, s10, 0xd
   12190: 00f867b3     	or	a5, a6, a5
   12194: 0116d813     	srli	a6, a3, 0x11
   12198: 00f69893     	slli	a7, a3, 0xf
   1219c: 0108e833     	or	a6, a7, a6
   121a0: 0136d893     	srli	a7, a3, 0x13
   121a4: 00d69393     	slli	t2, a3, 0xd
   121a8: 0113e8b3     	or	a7, t2, a7
   121ac: 0076d393     	srli	t2, a3, 0x7
   121b0: 01969e93     	slli	t4, a3, 0x19
   121b4: 007ee3b3     	or	t2, t4, t2
   121b8: 0126de93     	srli	t4, a3, 0x12
   121bc: 00e69413     	slli	s0, a3, 0xe
   121c0: 01d46eb3     	or	t4, s0, t4
   121c4: 007d5413     	srli	s0, s10, 0x7
   121c8: 019d1493     	slli	s1, s10, 0x19
   121cc: 0084e433     	or	s0, s1, s0
   121d0: 012d5493     	srli	s1, s10, 0x12
   121d4: 00ed1913     	slli	s2, s10, 0xe
   121d8: 009964b3     	or	s1, s2, s1
   121dc: 00f74733     	xor	a4, a4, a5
   121e0: 01184933     	xor	s2, a6, a7
   121e4: 01d3c8b3     	xor	a7, t2, t4
   121e8: 009447b3     	xor	a5, s0, s1
   121ec: 0d012803     	lw	a6, 0xd0(sp)
   121f0: 00b80833     	add	a6, a6, a1
   121f4: 00c30c33     	add	s8, t1, a2
   121f8: 0f412283     	lw	t0, 0xf4(sp)
   121fc: 00560633     	add	a2, a2, t0
   12200: 00b54333     	xor	t1, a0, a1
   12204: 00667333     	and	t1, a2, t1
   12208: 00b345b3     	xor	a1, t1, a1
   1220c: 00665313     	srli	t1, a2, 0x6
   12210: 01a61393     	slli	t2, a2, 0x1a
   12214: 0063e333     	or	t1, t2, t1
   12218: 00b65393     	srli	t2, a2, 0xb
   1221c: 01561e93     	slli	t4, a2, 0x15
   12220: 007ee3b3     	or	t2, t4, t2
   12224: 01965e93     	srli	t4, a2, 0x19
   12228: 00761413     	slli	s0, a2, 0x7
   1222c: 01d46eb3     	or	t4, s0, t4
   12230: 002c5413     	srli	s0, s8, 0x2
   12234: 01ec1493     	slli	s1, s8, 0x1e
   12238: 0084e433     	or	s0, s1, s0
   1223c: 00dc5493     	srli	s1, s8, 0xd
   12240: 013c1a93     	slli	s5, s8, 0x13
   12244: 009aeab3     	or	s5, s5, s1
   12248: 016c5493     	srli	s1, s8, 0x16
   1224c: 00ac1b13     	slli	s6, s8, 0xa
   12250: 009b6b33     	or	s6, s6, s1
   12254: 01e0c4b3     	xor	s1, ra, t5
   12258: 009c74b3     	and	s1, s8, s1
   1225c: 01e0fbb3     	and	s7, ra, t5
   12260: 0174cbb3     	xor	s7, s1, s7
   12264: 00ad5493     	srli	s1, s10, 0xa
   12268: 00974733     	xor	a4, a4, s1
   1226c: 0ed12623     	sw	a3, 0xec(sp)
   12270: 00a6d493     	srli	s1, a3, 0xa
   12274: 00994933     	xor	s2, s2, s1
   12278: 0036d493     	srli	s1, a3, 0x3
   1227c: 0098c6b3     	xor	a3, a7, s1
   12280: 06d12823     	sw	a3, 0x70(sp)
   12284: 003d5493     	srli	s1, s10, 0x3
   12288: 09a12a23     	sw	s10, 0x94(sp)
   1228c: 0097c7b3     	xor	a5, a5, s1
   12290: 04f12c23     	sw	a5, 0x58(sp)
   12294: 00bf85b3     	add	a1, t6, a1
   12298: 007347b3     	xor	a5, t1, t2
   1229c: 01544333     	xor	t1, s0, s5
   122a0: 0c812a03     	lw	s4, 0xc8(sp)
   122a4: 09012683     	lw	a3, 0x90(sp)
   122a8: 01468a33     	add	s4, a3, s4
   122ac: 0e412683     	lw	a3, 0xe4(sp)
   122b0: 00da0a33     	add	s4, s4, a3
   122b4: 00ea08b3     	add	a7, s4, a4
   122b8: 0bc12c83     	lw	s9, 0xbc(sp)
   122bc: 06c12683     	lw	a3, 0x6c(sp)
   122c0: 01968cb3     	add	s9, a3, s9
   122c4: 0dc12683     	lw	a3, 0xdc(sp)
   122c8: 00dc8cb3     	add	s9, s9, a3
   122cc: 012c86b3     	add	a3, s9, s2
   122d0: 01d7c733     	xor	a4, a5, t4
   122d4: 3956c7b7     	lui	a5, 0x3956c
   122d8: 25b78793     	addi	a5, a5, 0x25b
   122dc: 00f585b3     	add	a1, a1, a5
   122e0: 016347b3     	xor	a5, t1, s6
   122e4: 00e58733     	add	a4, a1, a4
   122e8: 017787b3     	add	a5, a5, s7
   122ec: 0118d593     	srli	a1, a7, 0x11
   122f0: 00f89313     	slli	t1, a7, 0xf
   122f4: 00b36333     	or	t1, t1, a1
   122f8: 0138d593     	srli	a1, a7, 0x13
   122fc: 00d89393     	slli	t2, a7, 0xd
   12300: 00b3e3b3     	or	t2, t2, a1
   12304: 0116d593     	srli	a1, a3, 0x11
   12308: 00f69e93     	slli	t4, a3, 0xf
   1230c: 00beeeb3     	or	t4, t4, a1
   12310: 0136d593     	srli	a1, a3, 0x13
   12314: 00d69f93     	slli	t6, a3, 0xd
   12318: 00bfefb3     	or	t6, t6, a1
   1231c: 0078d593     	srli	a1, a7, 0x7
   12320: 01989413     	slli	s0, a7, 0x19
   12324: 00b46433     	or	s0, s0, a1
   12328: 0128d593     	srli	a1, a7, 0x12
   1232c: 00e89913     	slli	s2, a7, 0xe
   12330: 00b96933     	or	s2, s2, a1
   12334: 0076d593     	srli	a1, a3, 0x7
   12338: 01969a13     	slli	s4, a3, 0x19
   1233c: 00ba6a33     	or	s4, s4, a1
   12340: 0126d593     	srli	a1, a3, 0x12
   12344: 00e69a93     	slli	s5, a3, 0xe
   12348: 00baeab3     	or	s5, s5, a1
   1234c: 01c705b3     	add	a1, a4, t3
   12350: 00e78db3     	add	s11, a5, a4
   12354: 00734733     	xor	a4, t1, t2
   12358: 01fec7b3     	xor	a5, t4, t6
   1235c: 01244333     	xor	t1, s0, s2
   12360: 015a43b3     	xor	t2, s4, s5
   12364: 00a8de13     	srli	t3, a7, 0xa
   12368: 01c74eb3     	xor	t4, a4, t3
   1236c: 0cd12823     	sw	a3, 0xd0(sp)
   12370: 00a6d713     	srli	a4, a3, 0xa
   12374: 00e7c7b3     	xor	a5, a5, a4
   12378: 0038d713     	srli	a4, a7, 0x3
   1237c: 00088e13     	mv	t3, a7
   12380: 09112823     	sw	a7, 0x90(sp)
   12384: 00e34733     	xor	a4, t1, a4
   12388: 04e12623     	sw	a4, 0x4c(sp)
   1238c: 0036d713     	srli	a4, a3, 0x3
   12390: 00e3c6b3     	xor	a3, t2, a4
   12394: 06d12623     	sw	a3, 0x6c(sp)
   12398: 0e012383     	lw	t2, 0xe0(sp)
   1239c: 00a383b3     	add	t2, t2, a0
   123a0: 00a64733     	xor	a4, a2, a0
   123a4: 00e5f733     	and	a4, a1, a4
   123a8: 00a74533     	xor	a0, a4, a0
   123ac: 0065d713     	srli	a4, a1, 0x6
   123b0: 01a59313     	slli	t1, a1, 0x1a
   123b4: 00e36733     	or	a4, t1, a4
   123b8: 00b5d313     	srli	t1, a1, 0xb
   123bc: 01559f93     	slli	t6, a1, 0x15
   123c0: 006fe333     	or	t1, t6, t1
   123c4: 0195df93     	srli	t6, a1, 0x19
   123c8: 00759413     	slli	s0, a1, 0x7
   123cc: 01f46fb3     	or	t6, s0, t6
   123d0: 002dd413     	srli	s0, s11, 0x2
   123d4: 01ed9913     	slli	s2, s11, 0x1e
   123d8: 00896433     	or	s0, s2, s0
   123dc: 00ddd913     	srli	s2, s11, 0xd
   123e0: 013d9a13     	slli	s4, s11, 0x13
   123e4: 012a6933     	or	s2, s4, s2
   123e8: 016dda13     	srli	s4, s11, 0x16
   123ec: 00ad9a93     	slli	s5, s11, 0xa
   123f0: 014aea33     	or	s4, s5, s4
   123f4: 001c4ab3     	xor	s5, s8, ra
   123f8: 015dfab3     	and	s5, s11, s5
   123fc: 001c7b33     	and	s6, s8, ra
   12400: 016acab3     	xor	s5, s5, s6
   12404: 07412b03     	lw	s6, 0x74(sp)
   12408: 08c12683     	lw	a3, 0x8c(sp)
   1240c: 01668b33     	add	s6, a3, s6
   12410: 0c412683     	lw	a3, 0xc4(sp)
   12414: 00db0b33     	add	s6, s6, a3
   12418: 01db04b3     	add	s1, s6, t4
   1241c: 0b812683     	lw	a3, 0xb8(sp)
   12420: 04812883     	lw	a7, 0x48(sp)
   12424: 00d886b3     	add	a3, a7, a3
   12428: 013686b3     	add	a3, a3, s3
   1242c: 00f688b3     	add	a7, a3, a5
   12430: 00a80533     	add	a0, a6, a0
   12434: 006746b3     	xor	a3, a4, t1
   12438: 01244733     	xor	a4, s0, s2
   1243c: 01f6c6b3     	xor	a3, a3, t6
   12440: 59f117b7     	lui	a5, 0x59f11
   12444: 1f178793     	addi	a5, a5, 0x1f1
   12448: 00f50533     	add	a0, a0, a5
   1244c: 01474733     	xor	a4, a4, s4
   12450: 0114d793     	srli	a5, s1, 0x11
   12454: 00f49813     	slli	a6, s1, 0xf
   12458: 00f867b3     	or	a5, a6, a5
   1245c: 0134d813     	srli	a6, s1, 0x13
   12460: 00d49313     	slli	t1, s1, 0xd
   12464: 01036833     	or	a6, t1, a6
   12468: 0118d313     	srli	t1, a7, 0x11
   1246c: 00f89e93     	slli	t4, a7, 0xf
   12470: 006ee333     	or	t1, t4, t1
   12474: 0138de93     	srli	t4, a7, 0x13
   12478: 00d89f93     	slli	t6, a7, 0xd
   1247c: 01dfeeb3     	or	t4, t6, t4
   12480: 0074df93     	srli	t6, s1, 0x7
   12484: 01949413     	slli	s0, s1, 0x19
   12488: 01f46fb3     	or	t6, s0, t6
   1248c: 0124d413     	srli	s0, s1, 0x12
   12490: 00e49913     	slli	s2, s1, 0xe
   12494: 00896433     	or	s0, s2, s0
   12498: 0078d913     	srli	s2, a7, 0x7
   1249c: 01989a13     	slli	s4, a7, 0x19
   124a0: 012a6933     	or	s2, s4, s2
   124a4: 0128da13     	srli	s4, a7, 0x12
   124a8: 00e89b13     	slli	s6, a7, 0xe
   124ac: 0b112a23     	sw	a7, 0xb4(sp)
   124b0: 014b6a33     	or	s4, s6, s4
   124b4: 00d50533     	add	a0, a0, a3
   124b8: 01570733     	add	a4, a4, s5
   124bc: 0107c6b3     	xor	a3, a5, a6
   124c0: 01d347b3     	xor	a5, t1, t4
   124c4: 008fc833     	xor	a6, t6, s0
   124c8: 01494333     	xor	t1, s2, s4
   124cc: 01e50b33     	add	s6, a0, t5
   124d0: 00a70f33     	add	t5, a4, a0
   124d4: 00a4d513     	srli	a0, s1, 0xa
   124d8: 00a6c533     	xor	a0, a3, a0
   124dc: 00a8d693     	srli	a3, a7, 0xa
   124e0: 00d7c6b3     	xor	a3, a5, a3
   124e4: 0034d713     	srli	a4, s1, 0x3
   124e8: 08912623     	sw	s1, 0x8c(sp)
   124ec: 00e84733     	xor	a4, a6, a4
   124f0: 02e12423     	sw	a4, 0x28(sp)
   124f4: 0038d713     	srli	a4, a7, 0x3
   124f8: 00e34733     	xor	a4, t1, a4
   124fc: 02e12823     	sw	a4, 0x30(sp)
   12500: 0ac12703     	lw	a4, 0xac(sp)
   12504: 08412783     	lw	a5, 0x84(sp)
   12508: 00e78733     	add	a4, a5, a4
   1250c: 0d412783     	lw	a5, 0xd4(sp)
   12510: 00f70733     	add	a4, a4, a5
   12514: 00a707b3     	add	a5, a4, a0
   12518: 0c012503     	lw	a0, 0xc0(sp)
   1251c: 06812703     	lw	a4, 0x68(sp)
   12520: 00a70533     	add	a0, a4, a0
   12524: 01a50533     	add	a0, a0, s10
   12528: 00d508b3     	add	a7, a0, a3
   1252c: 0e812503     	lw	a0, 0xe8(sp)
   12530: 00c50533     	add	a0, a0, a2
   12534: 04a12423     	sw	a0, 0x48(sp)
   12538: 00c5c533     	xor	a0, a1, a2
   1253c: 00ab7533     	and	a0, s6, a0
   12540: 00c54633     	xor	a2, a0, a2
   12544: 006b5513     	srli	a0, s6, 0x6
   12548: 01ab1693     	slli	a3, s6, 0x1a
   1254c: 00a6e6b3     	or	a3, a3, a0
   12550: 00bb5513     	srli	a0, s6, 0xb
   12554: 015b1713     	slli	a4, s6, 0x15
   12558: 00a76733     	or	a4, a4, a0
   1255c: 019b5513     	srli	a0, s6, 0x19
   12560: 007b1813     	slli	a6, s6, 0x7
   12564: 00a86533     	or	a0, a6, a0
   12568: 002f5813     	srli	a6, t5, 0x2
   1256c: 01ef1313     	slli	t1, t5, 0x1e
   12570: 01036833     	or	a6, t1, a6
   12574: 00df5313     	srli	t1, t5, 0xd
   12578: 013f1f93     	slli	t6, t5, 0x13
   1257c: 006fe333     	or	t1, t6, t1
   12580: 016f5f93     	srli	t6, t5, 0x16
   12584: 00af1413     	slli	s0, t5, 0xa
   12588: 01f46fb3     	or	t6, s0, t6
   1258c: 018dc433     	xor	s0, s11, s8
   12590: 008f7433     	and	s0, t5, s0
   12594: 018df933     	and	s2, s11, s8
   12598: 01244433     	xor	s0, s0, s2
   1259c: 00c38633     	add	a2, t2, a2
   125a0: 00e6c6b3     	xor	a3, a3, a4
   125a4: 00684733     	xor	a4, a6, t1
   125a8: 0117d813     	srli	a6, a5, 0x11
   125ac: 00f79313     	slli	t1, a5, 0xf
   125b0: 01036833     	or	a6, t1, a6
   125b4: 0137d313     	srli	t1, a5, 0x13
   125b8: 00d79393     	slli	t2, a5, 0xd
   125bc: 0063e333     	or	t1, t2, t1
   125c0: 0118d393     	srli	t2, a7, 0x11
   125c4: 00f89913     	slli	s2, a7, 0xf
   125c8: 007963b3     	or	t2, s2, t2
   125cc: 0138d913     	srli	s2, a7, 0x13
   125d0: 00d89a13     	slli	s4, a7, 0xd
   125d4: 012a6933     	or	s2, s4, s2
   125d8: 0077da13     	srli	s4, a5, 0x7
   125dc: 01979a93     	slli	s5, a5, 0x19
   125e0: 014aea33     	or	s4, s5, s4
   125e4: 0127da93     	srli	s5, a5, 0x12
   125e8: 00e79b93     	slli	s7, a5, 0xe
   125ec: 015beab3     	or	s5, s7, s5
   125f0: 0078db93     	srli	s7, a7, 0x7
   125f4: 01989c93     	slli	s9, a7, 0x19
   125f8: 017cebb3     	or	s7, s9, s7
   125fc: 0128dc93     	srli	s9, a7, 0x12
   12600: 00e89d13     	slli	s10, a7, 0xe
   12604: 019d6cb3     	or	s9, s10, s9
   12608: 00a6c533     	xor	a0, a3, a0
   1260c: 923f86b7     	lui	a3, 0x923f8
   12610: 2a468693     	addi	a3, a3, 0x2a4
   12614: 00d60633     	add	a2, a2, a3
   12618: 01f746b3     	xor	a3, a4, t6
   1261c: 00684733     	xor	a4, a6, t1
   12620: 0123c833     	xor	a6, t2, s2
   12624: 015a4333     	xor	t1, s4, s5
   12628: 019bc3b3     	xor	t2, s7, s9
   1262c: 00a60533     	add	a0, a2, a0
   12630: 008686b3     	add	a3, a3, s0
   12634: 00078c93     	mv	s9, a5
   12638: 00a7d613     	srli	a2, a5, 0xa
   1263c: 00c74633     	xor	a2, a4, a2
   12640: 07112423     	sw	a7, 0x68(sp)
   12644: 00a8d713     	srli	a4, a7, 0xa
   12648: 00e84733     	xor	a4, a6, a4
   1264c: 0037d813     	srli	a6, a5, 0x3
   12650: 08f12223     	sw	a5, 0x84(sp)
   12654: 010347b3     	xor	a5, t1, a6
   12658: 02f12623     	sw	a5, 0x2c(sp)
   1265c: 0038d813     	srli	a6, a7, 0x3
   12660: 0103c7b3     	xor	a5, t2, a6
   12664: 04f12023     	sw	a5, 0x40(sp)
   12668: 001503b3     	add	t2, a0, ra
   1266c: 00a68fb3     	add	t6, a3, a0
   12670: 07812503     	lw	a0, 0x78(sp)
   12674: 0b012683     	lw	a3, 0xb0(sp)
   12678: 00a68533     	add	a0, a3, a0
   1267c: 0ec12683     	lw	a3, 0xec(sp)
   12680: 00d50533     	add	a0, a0, a3
   12684: 00c502b3     	add	t0, a0, a2
   12688: 0d812503     	lw	a0, 0xd8(sp)
   1268c: 05012603     	lw	a2, 0x50(sp)
   12690: 00a60533     	add	a0, a2, a0
   12694: 01c50533     	add	a0, a0, t3
   12698: 00e508b3     	add	a7, a0, a4
   1269c: 0c812503     	lw	a0, 0xc8(sp)
   126a0: 00b50533     	add	a0, a0, a1
   126a4: 0aa12823     	sw	a0, 0xb0(sp)
   126a8: 00bb4533     	xor	a0, s6, a1
   126ac: 00a3f533     	and	a0, t2, a0
   126b0: 00b547b3     	xor	a5, a0, a1
   126b4: 0063d593     	srli	a1, t2, 0x6
   126b8: 01a39613     	slli	a2, t2, 0x1a
   126bc: 00b665b3     	or	a1, a2, a1
   126c0: 00b3d613     	srli	a2, t2, 0xb
   126c4: 01539693     	slli	a3, t2, 0x15
   126c8: 00c6e6b3     	or	a3, a3, a2
   126cc: 0193d613     	srli	a2, t2, 0x19
   126d0: 00739713     	slli	a4, t2, 0x7
   126d4: 00c76633     	or	a2, a4, a2
   126d8: 002fd713     	srli	a4, t6, 0x2
   126dc: 01ef9813     	slli	a6, t6, 0x1e
   126e0: 00e86733     	or	a4, a6, a4
   126e4: 00dfd813     	srli	a6, t6, 0xd
   126e8: 013f9313     	slli	t1, t6, 0x13
   126ec: 01036833     	or	a6, t1, a6
   126f0: 016fd313     	srli	t1, t6, 0x16
   126f4: 00af9413     	slli	s0, t6, 0xa
   126f8: 00646eb3     	or	t4, s0, t1
   126fc: 01bf4433     	xor	s0, t5, s11
   12700: 008ff433     	and	s0, t6, s0
   12704: 01bf7933     	and	s2, t5, s11
   12708: 01244433     	xor	s0, s0, s2
   1270c: 00028313     	mv	t1, t0
   12710: 0072d913     	srli	s2, t0, 0x7
   12714: 01929a13     	slli	s4, t0, 0x19
   12718: 012a6933     	or	s2, s4, s2
   1271c: 0122da13     	srli	s4, t0, 0x12
   12720: 00e29a93     	slli	s5, t0, 0xe
   12724: 014aea33     	or	s4, s5, s4
   12728: 0112da93     	srli	s5, t0, 0x11
   1272c: 00f29b93     	slli	s7, t0, 0xf
   12730: 015beab3     	or	s5, s7, s5
   12734: 0132db93     	srli	s7, t0, 0x13
   12738: 00d29d13     	slli	s10, t0, 0xd
   1273c: 017d6bb3     	or	s7, s10, s7
   12740: 00088513     	mv	a0, a7
   12744: 0118dd13     	srli	s10, a7, 0x11
   12748: 00f89093     	slli	ra, a7, 0xf
   1274c: 01a0ed33     	or	s10, ra, s10
   12750: 0138d093     	srli	ra, a7, 0x13
   12754: 00d89893     	slli	a7, a7, 0xd
   12758: 0018e8b3     	or	a7, a7, ra
   1275c: 00755093     	srli	ra, a0, 0x7
   12760: 01951293     	slli	t0, a0, 0x19
   12764: 0012e2b3     	or	t0, t0, ra
   12768: 01255093     	srli	ra, a0, 0x12
   1276c: 00e51e13     	slli	t3, a0, 0xe
   12770: 00050993     	mv	s3, a0
   12774: 001e6e33     	or	t3, t3, ra
   12778: 04812503     	lw	a0, 0x48(sp)
   1277c: 00f50533     	add	a0, a0, a5
   12780: 00d5c5b3     	xor	a1, a1, a3
   12784: 010746b3     	xor	a3, a4, a6
   12788: 01494733     	xor	a4, s2, s4
   1278c: 017ac7b3     	xor	a5, s5, s7
   12790: 011d4833     	xor	a6, s10, a7
   12794: 01c2c8b3     	xor	a7, t0, t3
   12798: 00c5c5b3     	xor	a1, a1, a2
   1279c: ab1c6637     	lui	a2, 0xab1c6
   127a0: ed560613     	addi	a2, a2, -0x12b
   127a4: 00c50533     	add	a0, a0, a2
   127a8: 01d6c633     	xor	a2, a3, t4
   127ac: 0e612023     	sw	t1, 0xe0(sp)
   127b0: 00335693     	srli	a3, t1, 0x3
   127b4: 00d746b3     	xor	a3, a4, a3
   127b8: 02d12c23     	sw	a3, 0x38(sp)
   127bc: 00a35693     	srli	a3, t1, 0xa
   127c0: 00d7c6b3     	xor	a3, a5, a3
   127c4: 0f312423     	sw	s3, 0xe8(sp)
   127c8: 00a9d713     	srli	a4, s3, 0xa
   127cc: 00e84733     	xor	a4, a6, a4
   127d0: 0039d793     	srli	a5, s3, 0x3
   127d4: 00f8c7b3     	xor	a5, a7, a5
   127d8: 02f12e23     	sw	a5, 0x3c(sp)
   127dc: 00b50533     	add	a0, a0, a1
   127e0: 00860633     	add	a2, a2, s0
   127e4: 0cc12583     	lw	a1, 0xcc(sp)
   127e8: 07c12783     	lw	a5, 0x7c(sp)
   127ec: 00b785b3     	add	a1, a5, a1
   127f0: 0d012783     	lw	a5, 0xd0(sp)
   127f4: 00f585b3     	add	a1, a1, a5
   127f8: 00d589b3     	add	s3, a1, a3
   127fc: 0e412583     	lw	a1, 0xe4(sp)
   12800: 05412683     	lw	a3, 0x54(sp)
   12804: 00b685b3     	add	a1, a3, a1
   12808: 009585b3     	add	a1, a1, s1
   1280c: 00e58eb3     	add	t4, a1, a4
   12810: 01850c33     	add	s8, a0, s8
   12814: 00a60533     	add	a0, a2, a0
   12818: 0119d593     	srli	a1, s3, 0x11
   1281c: 00f99613     	slli	a2, s3, 0xf
   12820: 00b66633     	or	a2, a2, a1
   12824: 0139d593     	srli	a1, s3, 0x13
   12828: 00d99693     	slli	a3, s3, 0xd
   1282c: 00b6e6b3     	or	a3, a3, a1
   12830: 011ed593     	srli	a1, t4, 0x11
   12834: 00fe9713     	slli	a4, t4, 0xf
   12838: 00b76733     	or	a4, a4, a1
   1283c: 013ed593     	srli	a1, t4, 0x13
   12840: 00de9793     	slli	a5, t4, 0xd
   12844: 00b7e7b3     	or	a5, a5, a1
   12848: 0079d593     	srli	a1, s3, 0x7
   1284c: 01999813     	slli	a6, s3, 0x19
   12850: 00b86833     	or	a6, a6, a1
   12854: 0129d593     	srli	a1, s3, 0x12
   12858: 00e99893     	slli	a7, s3, 0xe
   1285c: 00b8e333     	or	t1, a7, a1
   12860: 007ed593     	srli	a1, t4, 0x7
   12864: 019e9893     	slli	a7, t4, 0x19
   12868: 00b8e8b3     	or	a7, a7, a1
   1286c: 012ed593     	srli	a1, t4, 0x12
   12870: 00ee9293     	slli	t0, t4, 0xe
   12874: 00b2e2b3     	or	t0, t0, a1
   12878: 0bc12583     	lw	a1, 0xbc(sp)
   1287c: 016585b3     	add	a1, a1, s6
   12880: 0163ce33     	xor	t3, t2, s6
   12884: 01cc7e33     	and	t3, s8, t3
   12888: 016e4e33     	xor	t3, t3, s6
   1288c: 006c5413     	srli	s0, s8, 0x6
   12890: 01ac1913     	slli	s2, s8, 0x1a
   12894: 00896433     	or	s0, s2, s0
   12898: 00bc5913     	srli	s2, s8, 0xb
   1289c: 015c1a13     	slli	s4, s8, 0x15
   128a0: 012a6933     	or	s2, s4, s2
   128a4: 019c5a13     	srli	s4, s8, 0x19
   128a8: 007c1a93     	slli	s5, s8, 0x7
   128ac: 014aea33     	or	s4, s5, s4
   128b0: 00255a93     	srli	s5, a0, 0x2
   128b4: 01e51b13     	slli	s6, a0, 0x1e
   128b8: 015b6ab3     	or	s5, s6, s5
   128bc: 00d55b13     	srli	s6, a0, 0xd
   128c0: 01351b93     	slli	s7, a0, 0x13
   128c4: 016beb33     	or	s6, s7, s6
   128c8: 01655b93     	srli	s7, a0, 0x16
   128cc: 00a51d13     	slli	s10, a0, 0xa
   128d0: 017d6bb3     	or	s7, s10, s7
   128d4: 01efcd33     	xor	s10, t6, t5
   128d8: 01a57d33     	and	s10, a0, s10
   128dc: 01eff0b3     	and	ra, t6, t5
   128e0: 001d4d33     	xor	s10, s10, ra
   128e4: 00d64633     	xor	a2, a2, a3
   128e8: 00f74733     	xor	a4, a4, a5
   128ec: 006846b3     	xor	a3, a6, t1
   128f0: 0058c7b3     	xor	a5, a7, t0
   128f4: 0b012803     	lw	a6, 0xb0(sp)
   128f8: 01c80e33     	add	t3, a6, t3
   128fc: 01244833     	xor	a6, s0, s2
   12900: 016ac8b3     	xor	a7, s5, s6
   12904: 00098093     	mv	ra, s3
   12908: 00a9d293     	srli	t0, s3, 0xa
   1290c: 00564633     	xor	a2, a2, t0
   12910: 0dd12423     	sw	t4, 0xc8(sp)
   12914: 00aed293     	srli	t0, t4, 0xa
   12918: 00574733     	xor	a4, a4, t0
   1291c: 0039d293     	srli	t0, s3, 0x3
   12920: 07312e23     	sw	s3, 0x7c(sp)
   12924: 0056c6b3     	xor	a3, a3, t0
   12928: 02d12a23     	sw	a3, 0x34(sp)
   1292c: 003ed693     	srli	a3, t4, 0x3
   12930: 00d7c6b3     	xor	a3, a5, a3
   12934: 04d12a23     	sw	a3, 0x54(sp)
   12938: 014846b3     	xor	a3, a6, s4
   1293c: d807b7b7     	lui	a5, 0xd807b
   12940: a9878793     	addi	a5, a5, -0x568
   12944: 00fe07b3     	add	a5, t3, a5
   12948: 0178c833     	xor	a6, a7, s7
   1294c: 0dc12883     	lw	a7, 0xdc(sp)
   12950: 08012283     	lw	t0, 0x80(sp)
   12954: 011288b3     	add	a7, t0, a7
   12958: 0b412283     	lw	t0, 0xb4(sp)
   1295c: 005888b3     	add	a7, a7, t0
   12960: 00c88633     	add	a2, a7, a2
   12964: 05c12883     	lw	a7, 0x5c(sp)
   12968: 0c412983     	lw	s3, 0xc4(sp)
   1296c: 013889b3     	add	s3, a7, s3
   12970: 019989b3     	add	s3, s3, s9
   12974: 00e98eb3     	add	t4, s3, a4
   12978: 00d787b3     	add	a5, a5, a3
   1297c: 01a80833     	add	a6, a6, s10
   12980: 01b786b3     	add	a3, a5, s11
   12984: 00f80cb3     	add	s9, a6, a5
   12988: 00060793     	mv	a5, a2
   1298c: 01165613     	srli	a2, a2, 0x11
   12990: 00f79713     	slli	a4, a5, 0xf
   12994: 00c76633     	or	a2, a4, a2
   12998: 0137d713     	srli	a4, a5, 0x13
   1299c: 00d79813     	slli	a6, a5, 0xd
   129a0: 00e86433     	or	s0, a6, a4
   129a4: 011ed713     	srli	a4, t4, 0x11
   129a8: 00fe9813     	slli	a6, t4, 0xf
   129ac: 00e86733     	or	a4, a6, a4
   129b0: 013ed813     	srli	a6, t4, 0x13
   129b4: 00de9893     	slli	a7, t4, 0xd
   129b8: 0108e833     	or	a6, a7, a6
   129bc: 0077d893     	srli	a7, a5, 0x7
   129c0: 01979293     	slli	t0, a5, 0x19
   129c4: 0112e8b3     	or	a7, t0, a7
   129c8: 0127d293     	srli	t0, a5, 0x12
   129cc: 00e79313     	slli	t1, a5, 0xe
   129d0: 005362b3     	or	t0, t1, t0
   129d4: 007ed313     	srli	t1, t4, 0x7
   129d8: 019e9e13     	slli	t3, t4, 0x19
   129dc: 006e6e33     	or	t3, t3, t1
   129e0: 012ed313     	srli	t1, t4, 0x12
   129e4: 00ee9913     	slli	s2, t4, 0xe
   129e8: 00696933     	or	s2, s2, t1
   129ec: 00864433     	xor	s0, a2, s0
   129f0: 01074333     	xor	t1, a4, a6
   129f4: 0058c833     	xor	a6, a7, t0
   129f8: 012e4633     	xor	a2, t3, s2
   129fc: 07412703     	lw	a4, 0x74(sp)
   12a00: 00770733     	add	a4, a4, t2
   12a04: 007c48b3     	xor	a7, s8, t2
   12a08: 0116f8b3     	and	a7, a3, a7
   12a0c: 0078c3b3     	xor	t2, a7, t2
   12a10: 0066d893     	srli	a7, a3, 0x6
   12a14: 01a69293     	slli	t0, a3, 0x1a
   12a18: 0112e8b3     	or	a7, t0, a7
   12a1c: 00b6d293     	srli	t0, a3, 0xb
   12a20: 01569e13     	slli	t3, a3, 0x15
   12a24: 005e62b3     	or	t0, t3, t0
   12a28: 0196de13     	srli	t3, a3, 0x19
   12a2c: 00769913     	slli	s2, a3, 0x7
   12a30: 01c96933     	or	s2, s2, t3
   12a34: 002cde13     	srli	t3, s9, 0x2
   12a38: 01ec9993     	slli	s3, s9, 0x1e
   12a3c: 01c9ee33     	or	t3, s3, t3
   12a40: 00dcd993     	srli	s3, s9, 0xd
   12a44: 013c9a13     	slli	s4, s9, 0x13
   12a48: 013a69b3     	or	s3, s4, s3
   12a4c: 016cda13     	srli	s4, s9, 0x16
   12a50: 00ac9a93     	slli	s5, s9, 0xa
   12a54: 014aea33     	or	s4, s5, s4
   12a58: 00050b93     	mv	s7, a0
   12a5c: 01f54ab3     	xor	s5, a0, t6
   12a60: 015cfab3     	and	s5, s9, s5
   12a64: 01f57b33     	and	s6, a0, t6
   12a68: 016acab3     	xor	s5, s5, s6
   12a6c: 00a7db13     	srli	s6, a5, 0xa
   12a70: 01644433     	xor	s0, s0, s6
   12a74: 0bd12823     	sw	t4, 0xb0(sp)
   12a78: 00aedb13     	srli	s6, t4, 0xa
   12a7c: 01634333     	xor	t1, t1, s6
   12a80: 0037db13     	srli	s6, a5, 0x3
   12a84: 08f12023     	sw	a5, 0x80(sp)
   12a88: 01684833     	xor	a6, a6, s6
   12a8c: 05012223     	sw	a6, 0x44(sp)
   12a90: 003ed813     	srli	a6, t4, 0x3
   12a94: 01064633     	xor	a2, a2, a6
   12a98: 04c12e23     	sw	a2, 0x5c(sp)
   12a9c: 007585b3     	add	a1, a1, t2
   12aa0: 0058c633     	xor	a2, a7, t0
   12aa4: 013e4833     	xor	a6, t3, s3
   12aa8: 08812883     	lw	a7, 0x88(sp)
   12aac: 06412503     	lw	a0, 0x64(sp)
   12ab0: 011508b3     	add	a7, a0, a7
   12ab4: 06812d03     	lw	s10, 0x68(sp)
   12ab8: 01a888b3     	add	a7, a7, s10
   12abc: 00888db3     	add	s11, a7, s0
   12ac0: 0d412483     	lw	s1, 0xd4(sp)
   12ac4: 05812503     	lw	a0, 0x58(sp)
   12ac8: 009504b3     	add	s1, a0, s1
   12acc: 0e012883     	lw	a7, 0xe0(sp)
   12ad0: 011484b3     	add	s1, s1, a7
   12ad4: 00648eb3     	add	t4, s1, t1
   12ad8: 01264633     	xor	a2, a2, s2
   12adc: 128368b7     	lui	a7, 0x12836
   12ae0: b0188893     	addi	a7, a7, -0x4ff
   12ae4: 011585b3     	add	a1, a1, a7
   12ae8: 01484833     	xor	a6, a6, s4
   12aec: 00c58633     	add	a2, a1, a2
   12af0: 01580833     	add	a6, a6, s5
   12af4: 011dd593     	srli	a1, s11, 0x11
   12af8: 00fd9893     	slli	a7, s11, 0xf
   12afc: 00b8e8b3     	or	a7, a7, a1
   12b00: 013dd593     	srli	a1, s11, 0x13
   12b04: 00dd9293     	slli	t0, s11, 0xd
   12b08: 00b2e2b3     	or	t0, t0, a1
   12b0c: 011ed593     	srli	a1, t4, 0x11
   12b10: 00fe9313     	slli	t1, t4, 0xf
   12b14: 00b36333     	or	t1, t1, a1
   12b18: 013ed593     	srli	a1, t4, 0x13
   12b1c: 00de9393     	slli	t2, t4, 0xd
   12b20: 00b3e3b3     	or	t2, t2, a1
   12b24: 007dd593     	srli	a1, s11, 0x7
   12b28: 019d9e13     	slli	t3, s11, 0x19
   12b2c: 00be6e33     	or	t3, t3, a1
   12b30: 012dd593     	srli	a1, s11, 0x12
   12b34: 00ed9413     	slli	s0, s11, 0xe
   12b38: 00b46433     	or	s0, s0, a1
   12b3c: 007ed593     	srli	a1, t4, 0x7
   12b40: 019e9493     	slli	s1, t4, 0x19
   12b44: 00b4e4b3     	or	s1, s1, a1
   12b48: 012ed593     	srli	a1, t4, 0x12
   12b4c: 00ee9913     	slli	s2, t4, 0xe
   12b50: 00b96933     	or	s2, s2, a1
   12b54: 01e605b3     	add	a1, a2, t5
   12b58: 00c80633     	add	a2, a6, a2
   12b5c: 0058c833     	xor	a6, a7, t0
   12b60: 007348b3     	xor	a7, t1, t2
   12b64: 008e43b3     	xor	t2, t3, s0
   12b68: 0124ce33     	xor	t3, s1, s2
   12b6c: 00add293     	srli	t0, s11, 0xa
   12b70: 00584333     	xor	t1, a6, t0
   12b74: 0bd12e23     	sw	t4, 0xbc(sp)
   12b78: 00aed813     	srli	a6, t4, 0xa
   12b7c: 0108c2b3     	xor	t0, a7, a6
   12b80: 003dd813     	srli	a6, s11, 0x3
   12b84: 07b12a23     	sw	s11, 0x74(sp)
   12b88: 0103c833     	xor	a6, t2, a6
   12b8c: 05012823     	sw	a6, 0x50(sp)
   12b90: 003ed813     	srli	a6, t4, 0x3
   12b94: 010e4833     	xor	a6, t3, a6
   12b98: 05012c23     	sw	a6, 0x58(sp)
   12b9c: 0b812803     	lw	a6, 0xb8(sp)
   12ba0: 01880833     	add	a6, a6, s8
   12ba4: 0186c8b3     	xor	a7, a3, s8
   12ba8: 0115f8b3     	and	a7, a1, a7
   12bac: 0188c8b3     	xor	a7, a7, s8
   12bb0: 0065d393     	srli	t2, a1, 0x6
   12bb4: 01a59e13     	slli	t3, a1, 0x1a
   12bb8: 007e63b3     	or	t2, t3, t2
   12bbc: 00b5de13     	srli	t3, a1, 0xb
   12bc0: 01559f13     	slli	t5, a1, 0x15
   12bc4: 01cf6e33     	or	t3, t5, t3
   12bc8: 0195df13     	srli	t5, a1, 0x19
   12bcc: 00759413     	slli	s0, a1, 0x7
   12bd0: 01e46f33     	or	t5, s0, t5
   12bd4: 00265413     	srli	s0, a2, 0x2
   12bd8: 01e61493     	slli	s1, a2, 0x1e
   12bdc: 0084e433     	or	s0, s1, s0
   12be0: 00d65493     	srli	s1, a2, 0xd
   12be4: 01361913     	slli	s2, a2, 0x13
   12be8: 009964b3     	or	s1, s2, s1
   12bec: 01665913     	srli	s2, a2, 0x16
   12bf0: 00a61993     	slli	s3, a2, 0xa
   12bf4: 0129e933     	or	s2, s3, s2
   12bf8: 017cc9b3     	xor	s3, s9, s7
   12bfc: 013679b3     	and	s3, a2, s3
   12c00: 017cfa33     	and	s4, s9, s7
   12c04: 0149c9b3     	xor	s3, s3, s4
   12c08: 09412a03     	lw	s4, 0x94(sp)
   12c0c: 07012503     	lw	a0, 0x70(sp)
   12c10: 01450a33     	add	s4, a0, s4
   12c14: 0e812e83     	lw	t4, 0xe8(sp)
   12c18: 01da0a33     	add	s4, s4, t4
   12c1c: 006a0c33     	add	s8, s4, t1
   12c20: 0ec12303     	lw	t1, 0xec(sp)
   12c24: 04c12503     	lw	a0, 0x4c(sp)
   12c28: 00650333     	add	t1, a0, t1
   12c2c: 00130333     	add	t1, t1, ra
   12c30: 00530eb3     	add	t4, t1, t0
   12c34: 01170733     	add	a4, a4, a7
   12c38: 01c3c8b3     	xor	a7, t2, t3
   12c3c: 00944433     	xor	s0, s0, s1
   12c40: 01e8c8b3     	xor	a7, a7, t5
   12c44: 243182b7     	lui	t0, 0x24318
   12c48: 5be28293     	addi	t0, t0, 0x5be
   12c4c: 00570733     	add	a4, a4, t0
   12c50: 012442b3     	xor	t0, s0, s2
   12c54: 011ed313     	srli	t1, t4, 0x11
   12c58: 00fe9393     	slli	t2, t4, 0xf
   12c5c: 0063e333     	or	t1, t2, t1
   12c60: 013ed393     	srli	t2, t4, 0x13
   12c64: 00de9e13     	slli	t3, t4, 0xd
   12c68: 007e63b3     	or	t2, t3, t2
   12c6c: 011c5e13     	srli	t3, s8, 0x11
   12c70: 00fc1f13     	slli	t5, s8, 0xf
   12c74: 01cf6e33     	or	t3, t5, t3
   12c78: 013c5f13     	srli	t5, s8, 0x13
   12c7c: 00dc1413     	slli	s0, s8, 0xd
   12c80: 01e46f33     	or	t5, s0, t5
   12c84: 007c5413     	srli	s0, s8, 0x7
   12c88: 019c1493     	slli	s1, s8, 0x19
   12c8c: 0084e433     	or	s0, s1, s0
   12c90: 012c5493     	srli	s1, s8, 0x12
   12c94: 00ec1913     	slli	s2, s8, 0xe
   12c98: 009964b3     	or	s1, s2, s1
   12c9c: 007ed913     	srli	s2, t4, 0x7
   12ca0: 019e9a13     	slli	s4, t4, 0x19
   12ca4: 012a6933     	or	s2, s4, s2
   12ca8: 012eda13     	srli	s4, t4, 0x12
   12cac: 00ee9a93     	slli	s5, t4, 0xe
   12cb0: 014aea33     	or	s4, s5, s4
   12cb4: 01170733     	add	a4, a4, a7
   12cb8: 013282b3     	add	t0, t0, s3
   12cbc: 007348b3     	xor	a7, t1, t2
   12cc0: 01ee4333     	xor	t1, t3, t5
   12cc4: 00944433     	xor	s0, s0, s1
   12cc8: 014943b3     	xor	t2, s2, s4
   12ccc: 01f709b3     	add	s3, a4, t6
   12cd0: 00e28e33     	add	t3, t0, a4
   12cd4: 0bd12c23     	sw	t4, 0xb8(sp)
   12cd8: 00aed713     	srli	a4, t4, 0xa
   12cdc: 00e8c733     	xor	a4, a7, a4
   12ce0: 00ac5893     	srli	a7, s8, 0xa
   12ce4: 011348b3     	xor	a7, t1, a7
   12ce8: 003c5293     	srli	t0, s8, 0x3
   12cec: 07812823     	sw	s8, 0x70(sp)
   12cf0: 00544533     	xor	a0, s0, t0
   12cf4: 04a12423     	sw	a0, 0x48(sp)
   12cf8: 003ed293     	srli	t0, t4, 0x3
   12cfc: 0053c533     	xor	a0, t2, t0
   12d00: 04a12623     	sw	a0, 0x4c(sp)
   12d04: 0d012e83     	lw	t4, 0xd0(sp)
   12d08: 02812503     	lw	a0, 0x28(sp)
   12d0c: 01d50eb3     	add	t4, a0, t4
   12d10: 00fe8eb3     	add	t4, t4, a5
   12d14: 00ee8933     	add	s2, t4, a4
   12d18: 09012703     	lw	a4, 0x90(sp)
   12d1c: 06c12503     	lw	a0, 0x6c(sp)
   12d20: 00e50733     	add	a4, a0, a4
   12d24: 0c812503     	lw	a0, 0xc8(sp)
   12d28: 00a70733     	add	a4, a4, a0
   12d2c: 01170a33     	add	s4, a4, a7
   12d30: 0ac12083     	lw	ra, 0xac(sp)
   12d34: 00d080b3     	add	ra, ra, a3
   12d38: 00d5c733     	xor	a4, a1, a3
   12d3c: 00e9f733     	and	a4, s3, a4
   12d40: 00d74733     	xor	a4, a4, a3
   12d44: 0069d693     	srli	a3, s3, 0x6
   12d48: 01a99893     	slli	a7, s3, 0x1a
   12d4c: 00d8e8b3     	or	a7, a7, a3
   12d50: 00b9d693     	srli	a3, s3, 0xb
   12d54: 01599293     	slli	t0, s3, 0x15
   12d58: 00d2e333     	or	t1, t0, a3
   12d5c: 0199d693     	srli	a3, s3, 0x19
   12d60: 00799293     	slli	t0, s3, 0x7
   12d64: 00d2e2b3     	or	t0, t0, a3
   12d68: 000e0513     	mv	a0, t3
   12d6c: 002e5693     	srli	a3, t3, 0x2
   12d70: 01ee1393     	slli	t2, t3, 0x1e
   12d74: 00d3e3b3     	or	t2, t2, a3
   12d78: 00de5693     	srli	a3, t3, 0xd
   12d7c: 013e1e13     	slli	t3, t3, 0x13
   12d80: 00de6e33     	or	t3, t3, a3
   12d84: 01655693     	srli	a3, a0, 0x16
   12d88: 00a51e93     	slli	t4, a0, 0xa
   12d8c: 00deef33     	or	t5, t4, a3
   12d90: 019646b3     	xor	a3, a2, s9
   12d94: 00d576b3     	and	a3, a0, a3
   12d98: 00050793     	mv	a5, a0
   12d9c: 01967eb3     	and	t4, a2, s9
   12da0: 00060a93     	mv	s5, a2
   12da4: 01d6c6b3     	xor	a3, a3, t4
   12da8: 00e804b3     	add	s1, a6, a4
   12dac: 0068cfb3     	xor	t6, a7, t1
   12db0: 01c3c433     	xor	s0, t2, t3
   12db4: 01195713     	srli	a4, s2, 0x11
   12db8: 00f91813     	slli	a6, s2, 0xf
   12dbc: 00e86733     	or	a4, a6, a4
   12dc0: 01395813     	srli	a6, s2, 0x13
   12dc4: 00d91893     	slli	a7, s2, 0xd
   12dc8: 0108e833     	or	a6, a7, a6
   12dcc: 011a5893     	srli	a7, s4, 0x11
   12dd0: 00fa1313     	slli	t1, s4, 0xf
   12dd4: 011368b3     	or	a7, t1, a7
   12dd8: 013a5313     	srli	t1, s4, 0x13
   12ddc: 00da1393     	slli	t2, s4, 0xd
   12de0: 0063e3b3     	or	t2, t2, t1
   12de4: 00795313     	srli	t1, s2, 0x7
   12de8: 01991e13     	slli	t3, s2, 0x19
   12dec: 006e6333     	or	t1, t3, t1
   12df0: 01295e13     	srli	t3, s2, 0x12
   12df4: 00e91e93     	slli	t4, s2, 0xe
   12df8: 01ceeeb3     	or	t4, t4, t3
   12dfc: 005fce33     	xor	t3, t6, t0
   12e00: 550c82b7     	lui	t0, 0x550c8
   12e04: dc328293     	addi	t0, t0, -0x23d
   12e08: 005484b3     	add	s1, s1, t0
   12e0c: 01e442b3     	xor	t0, s0, t5
   12e10: 000a0613     	mv	a2, s4
   12e14: 007a5f13     	srli	t5, s4, 0x7
   12e18: 019a1f93     	slli	t6, s4, 0x19
   12e1c: 01efef33     	or	t5, t6, t5
   12e20: 012a5f93     	srli	t6, s4, 0x12
   12e24: 00ea1413     	slli	s0, s4, 0xe
   12e28: 01f46fb3     	or	t6, s0, t6
   12e2c: 01074733     	xor	a4, a4, a6
   12e30: 0078c833     	xor	a6, a7, t2
   12e34: 01d348b3     	xor	a7, t1, t4
   12e38: 01c483b3     	add	t2, s1, t3
   12e3c: 00d286b3     	add	a3, t0, a3
   12e40: 01ff42b3     	xor	t0, t5, t6
   12e44: 07212223     	sw	s2, 0x64(sp)
   12e48: 00a95313     	srli	t1, s2, 0xa
   12e4c: 00674b33     	xor	s6, a4, t1
   12e50: 00aa5713     	srli	a4, s4, 0xa
   12e54: 00e84433     	xor	s0, a6, a4
   12e58: 00395713     	srli	a4, s2, 0x3
   12e5c: 00e8c533     	xor	a0, a7, a4
   12e60: 02a12423     	sw	a0, 0x28(sp)
   12e64: 01738eb3     	add	t4, t2, s7
   12e68: 007683b3     	add	t2, a3, t2
   12e6c: 003a5513     	srli	a0, s4, 0x3
   12e70: 07412623     	sw	s4, 0x6c(sp)
   12e74: 00a2c533     	xor	a0, t0, a0
   12e78: 02a12223     	sw	a0, 0x24(sp)
   12e7c: 02c12503     	lw	a0, 0x2c(sp)
   12e80: 0b412683     	lw	a3, 0xb4(sp)
   12e84: 00d50533     	add	a0, a0, a3
   12e88: 01b50533     	add	a0, a0, s11
   12e8c: 016508b3     	add	a7, a0, s6
   12e90: 08c12503     	lw	a0, 0x8c(sp)
   12e94: 03012683     	lw	a3, 0x30(sp)
   12e98: 00a68533     	add	a0, a3, a0
   12e9c: 0b012683     	lw	a3, 0xb0(sp)
   12ea0: 00d50533     	add	a0, a0, a3
   12ea4: 00850433     	add	s0, a0, s0
   12ea8: 0c012503     	lw	a0, 0xc0(sp)
   12eac: 00b50533     	add	a0, a0, a1
   12eb0: 00b9c6b3     	xor	a3, s3, a1
   12eb4: 00def6b3     	and	a3, t4, a3
   12eb8: 00b6c4b3     	xor	s1, a3, a1
   12ebc: 006ed593     	srli	a1, t4, 0x6
   12ec0: 01ae9693     	slli	a3, t4, 0x1a
   12ec4: 00b6ee33     	or	t3, a3, a1
   12ec8: 00bed593     	srli	a1, t4, 0xb
   12ecc: 015e9693     	slli	a3, t4, 0x15
   12ed0: 00b6ef33     	or	t5, a3, a1
   12ed4: 019ed593     	srli	a1, t4, 0x19
   12ed8: 007e9693     	slli	a3, t4, 0x7
   12edc: 00b6e5b3     	or	a1, a3, a1
   12ee0: 0023d693     	srli	a3, t2, 0x2
   12ee4: 01e39713     	slli	a4, t2, 0x1e
   12ee8: 00d76933     	or	s2, a4, a3
   12eec: 00d3d693     	srli	a3, t2, 0xd
   12ef0: 01339713     	slli	a4, t2, 0x13
   12ef4: 00d76bb3     	or	s7, a4, a3
   12ef8: 0163d693     	srli	a3, t2, 0x16
   12efc: 00a39713     	slli	a4, t2, 0xa
   12f00: 00d76733     	or	a4, a4, a3
   12f04: 01512a23     	sw	s5, 0x14(sp)
   12f08: 0157c6b3     	xor	a3, a5, s5
   12f0c: 00d3f6b3     	and	a3, t2, a3
   12f10: 0157f833     	and	a6, a5, s5
   12f14: 00078b13     	mv	s6, a5
   12f18: 0106c6b3     	xor	a3, a3, a6
   12f1c: 00088793     	mv	a5, a7
   12f20: 0118d813     	srli	a6, a7, 0x11
   12f24: 00f89893     	slli	a7, a7, 0xf
   12f28: 0108e833     	or	a6, a7, a6
   12f2c: 0137d893     	srli	a7, a5, 0x13
   12f30: 00d79293     	slli	t0, a5, 0xd
   12f34: 0112e8b3     	or	a7, t0, a7
   12f38: 01145293     	srli	t0, s0, 0x11
   12f3c: 00f41313     	slli	t1, s0, 0xf
   12f40: 005362b3     	or	t0, t1, t0
   12f44: 01345313     	srli	t1, s0, 0x13
   12f48: 00d41f93     	slli	t6, s0, 0xd
   12f4c: 006fe333     	or	t1, t6, t1
   12f50: 0077df93     	srli	t6, a5, 0x7
   12f54: 01979a13     	slli	s4, a5, 0x19
   12f58: 01fa6fb3     	or	t6, s4, t6
   12f5c: 0127da13     	srli	s4, a5, 0x12
   12f60: 00e79a93     	slli	s5, a5, 0xe
   12f64: 014aea33     	or	s4, s5, s4
   12f68: 009084b3     	add	s1, ra, s1
   12f6c: 01ee4ab3     	xor	s5, t3, t5
   12f70: 01794933     	xor	s2, s2, s7
   12f74: 00745e13     	srli	t3, s0, 0x7
   12f78: 01941f13     	slli	t5, s0, 0x19
   12f7c: 01cf6e33     	or	t3, t5, t3
   12f80: 01245f13     	srli	t5, s0, 0x12
   12f84: 00e41b93     	slli	s7, s0, 0xe
   12f88: 01ebef33     	or	t5, s7, t5
   12f8c: 01184833     	xor	a6, a6, a7
   12f90: 0062c8b3     	xor	a7, t0, t1
   12f94: 014fc2b3     	xor	t0, t6, s4
   12f98: 00bac5b3     	xor	a1, s5, a1
   12f9c: 72be6337     	lui	t1, 0x72be6
   12fa0: d7430313     	addi	t1, t1, -0x28c
   12fa4: 00648333     	add	t1, s1, t1
   12fa8: 00e94733     	xor	a4, s2, a4
   12fac: 01ee4e33     	xor	t3, t3, t5
   12fb0: 00f12e23     	sw	a5, 0x1c(sp)
   12fb4: 00a7df13     	srli	t5, a5, 0xa
   12fb8: 01e84f33     	xor	t5, a6, t5
   12fbc: 00a45813     	srli	a6, s0, 0xa
   12fc0: 0108c8b3     	xor	a7, a7, a6
   12fc4: 0037d813     	srli	a6, a5, 0x3
   12fc8: 0102c7b3     	xor	a5, t0, a6
   12fcc: 02f12823     	sw	a5, 0x30(sp)
   12fd0: 00b305b3     	add	a1, t1, a1
   12fd4: 00d706b3     	add	a3, a4, a3
   12fd8: 00345713     	srli	a4, s0, 0x3
   12fdc: 0a812623     	sw	s0, 0xac(sp)
   12fe0: 00ee4733     	xor	a4, t3, a4
   12fe4: 02e12623     	sw	a4, 0x2c(sp)
   12fe8: 03812703     	lw	a4, 0x38(sp)
   12fec: 01a70733     	add	a4, a4, s10
   12ff0: 01870733     	add	a4, a4, s8
   12ff4: 01e70c33     	add	s8, a4, t5
   12ff8: 08412703     	lw	a4, 0x84(sp)
   12ffc: 04012783     	lw	a5, 0x40(sp)
   13000: 00e78733     	add	a4, a5, a4
   13004: 0bc12783     	lw	a5, 0xbc(sp)
   13008: 00f70733     	add	a4, a4, a5
   1300c: 01170f33     	add	t5, a4, a7
   13010: 019587b3     	add	a5, a1, s9
   13014: 00b68fb3     	add	t6, a3, a1
   13018: 007c5593     	srli	a1, s8, 0x7
   1301c: 019c1693     	slli	a3, s8, 0x19
   13020: 00b6e5b3     	or	a1, a3, a1
   13024: 012c5693     	srli	a3, s8, 0x12
   13028: 00ec1713     	slli	a4, s8, 0xe
   1302c: 00d76733     	or	a4, a4, a3
   13030: 011f5693     	srli	a3, t5, 0x11
   13034: 00ff1813     	slli	a6, t5, 0xf
   13038: 00d86833     	or	a6, a6, a3
   1303c: 013f5693     	srli	a3, t5, 0x13
   13040: 00df1293     	slli	t0, t5, 0xd
   13044: 00d2e2b3     	or	t0, t0, a3
   13048: 011c5693     	srli	a3, s8, 0x11
   1304c: 00fc1313     	slli	t1, s8, 0xf
   13050: 00d36333     	or	t1, t1, a3
   13054: 013c5693     	srli	a3, s8, 0x13
   13058: 00dc1e13     	slli	t3, s8, 0xd
   1305c: 00de6e33     	or	t3, t3, a3
   13060: 07812683     	lw	a3, 0x78(sp)
   13064: 013686b3     	add	a3, a3, s3
   13068: 013ec4b3     	xor	s1, t4, s3
   1306c: 0097f4b3     	and	s1, a5, s1
   13070: 0134c4b3     	xor	s1, s1, s3
   13074: 0067d913     	srli	s2, a5, 0x6
   13078: 01a79993     	slli	s3, a5, 0x1a
   1307c: 0129e933     	or	s2, s3, s2
   13080: 00b7d993     	srli	s3, a5, 0xb
   13084: 01579a13     	slli	s4, a5, 0x15
   13088: 013a69b3     	or	s3, s4, s3
   1308c: 0197da13     	srli	s4, a5, 0x19
   13090: 00779a93     	slli	s5, a5, 0x7
   13094: 014aea33     	or	s4, s5, s4
   13098: 002fda93     	srli	s5, t6, 0x2
   1309c: 01ef9b93     	slli	s7, t6, 0x1e
   130a0: 015beab3     	or	s5, s7, s5
   130a4: 00dfdb93     	srli	s7, t6, 0xd
   130a8: 013f9c93     	slli	s9, t6, 0x13
   130ac: 017cebb3     	or	s7, s9, s7
   130b0: 016fdc93     	srli	s9, t6, 0x16
   130b4: 00af9d13     	slli	s10, t6, 0xa
   130b8: 019d6cb3     	or	s9, s10, s9
   130bc: 0163cd33     	xor	s10, t2, s6
   130c0: 01affd33     	and	s10, t6, s10
   130c4: 0163fdb3     	and	s11, t2, s6
   130c8: 01bd4d33     	xor	s10, s10, s11
   130cc: 007f5d93     	srli	s11, t5, 0x7
   130d0: 019f1093     	slli	ra, t5, 0x19
   130d4: 01b0edb3     	or	s11, ra, s11
   130d8: 012f5093     	srli	ra, t5, 0x12
   130dc: 00ef1893     	slli	a7, t5, 0xe
   130e0: 0018e8b3     	or	a7, a7, ra
   130e4: 00e5c5b3     	xor	a1, a1, a4
   130e8: 00584733     	xor	a4, a6, t0
   130ec: 01c34833     	xor	a6, t1, t3
   130f0: 00950533     	add	a0, a0, s1
   130f4: 013942b3     	xor	t0, s2, s3
   130f8: 017ac333     	xor	t1, s5, s7
   130fc: 011dc8b3     	xor	a7, s11, a7
   13100: 01812c23     	sw	s8, 0x18(sp)
   13104: 003c5e13     	srli	t3, s8, 0x3
   13108: 01c5c5b3     	xor	a1, a1, t3
   1310c: 02b12c23     	sw	a1, 0x38(sp)
   13110: 00af5593     	srli	a1, t5, 0xa
   13114: 00b745b3     	xor	a1, a4, a1
   13118: 00ac5713     	srli	a4, s8, 0xa
   1311c: 00e84e33     	xor	t3, a6, a4
   13120: 0142c833     	xor	a6, t0, s4
   13124: 80deb737     	lui	a4, 0x80deb
   13128: 1fe70713     	addi	a4, a4, 0x1fe
   1312c: 00e50533     	add	a0, a0, a4
   13130: 01934333     	xor	t1, t1, s9
   13134: 003f5713     	srli	a4, t5, 0x3
   13138: 000f0293     	mv	t0, t5
   1313c: 0de12023     	sw	t5, 0xc0(sp)
   13140: 00e8c733     	xor	a4, a7, a4
   13144: 02e12023     	sw	a4, 0x20(sp)
   13148: 0e012883     	lw	a7, 0xe0(sp)
   1314c: 03c12703     	lw	a4, 0x3c(sp)
   13150: 011708b3     	add	a7, a4, a7
   13154: 0b812703     	lw	a4, 0xb8(sp)
   13158: 00e888b3     	add	a7, a7, a4
   1315c: 00b888b3     	add	a7, a7, a1
   13160: 0e812583     	lw	a1, 0xe8(sp)
   13164: 03412703     	lw	a4, 0x34(sp)
   13168: 00b705b3     	add	a1, a4, a1
   1316c: 00c585b3     	add	a1, a1, a2
   13170: 01c58e33     	add	t3, a1, t3
   13174: 01050533     	add	a0, a0, a6
   13178: 01a30333     	add	t1, t1, s10
   1317c: 01412b83     	lw	s7, 0x14(sp)
   13180: 01750bb3     	add	s7, a0, s7
   13184: 00a30a33     	add	s4, t1, a0
   13188: 011e5513     	srli	a0, t3, 0x11
   1318c: 00fe1593     	slli	a1, t3, 0xf
   13190: 00a5e533     	or	a0, a1, a0
   13194: 013e5593     	srli	a1, t3, 0x13
   13198: 00de1613     	slli	a2, t3, 0xd
   1319c: 00b665b3     	or	a1, a2, a1
   131a0: 0118d613     	srli	a2, a7, 0x11
   131a4: 00f89493     	slli	s1, a7, 0xf
   131a8: 00c4e4b3     	or	s1, s1, a2
   131ac: 0138d613     	srli	a2, a7, 0x13
   131b0: 00d89813     	slli	a6, a7, 0xd
   131b4: 00c869b3     	or	s3, a6, a2
   131b8: 00088713     	mv	a4, a7
   131bc: 0078d613     	srli	a2, a7, 0x7
   131c0: 01989813     	slli	a6, a7, 0x19
   131c4: 00c86633     	or	a2, a6, a2
   131c8: 0128d813     	srli	a6, a7, 0x12
   131cc: 00e89893     	slli	a7, a7, 0xe
   131d0: 00070f13     	mv	t5, a4
   131d4: 0108e833     	or	a6, a7, a6
   131d8: 007e5893     	srli	a7, t3, 0x7
   131dc: 019e1313     	slli	t1, t3, 0x19
   131e0: 011368b3     	or	a7, t1, a7
   131e4: 012e5313     	srli	t1, t3, 0x12
   131e8: 00ee1913     	slli	s2, t3, 0xe
   131ec: 00696333     	or	t1, s2, t1
   131f0: 00b54533     	xor	a0, a0, a1
   131f4: 0134c4b3     	xor	s1, s1, s3
   131f8: 0d812583     	lw	a1, 0xd8(sp)
   131fc: 01d585b3     	add	a1, a1, t4
   13200: 01d7c933     	xor	s2, a5, t4
   13204: 012bf933     	and	s2, s7, s2
   13208: 01d94eb3     	xor	t4, s2, t4
   1320c: 006bd913     	srli	s2, s7, 0x6
   13210: 01ab9993     	slli	s3, s7, 0x1a
   13214: 0129e933     	or	s2, s3, s2
   13218: 00bbd993     	srli	s3, s7, 0xb
   1321c: 015b9a93     	slli	s5, s7, 0x15
   13220: 013ae9b3     	or	s3, s5, s3
   13224: 019bda93     	srli	s5, s7, 0x19
   13228: 007b9c93     	slli	s9, s7, 0x7
   1322c: 015ceab3     	or	s5, s9, s5
   13230: 002a5c93     	srli	s9, s4, 0x2
   13234: 01ea1d13     	slli	s10, s4, 0x1e
   13238: 019d6cb3     	or	s9, s10, s9
   1323c: 00da5d13     	srli	s10, s4, 0xd
   13240: 013a1d93     	slli	s11, s4, 0x13
   13244: 01aded33     	or	s10, s11, s10
   13248: 016a5d93     	srli	s11, s4, 0x16
   1324c: 00aa1093     	slli	ra, s4, 0xa
   13250: 01b0edb3     	or	s11, ra, s11
   13254: 007fc0b3     	xor	ra, t6, t2
   13258: 001a70b3     	and	ra, s4, ra
   1325c: 007ff733     	and	a4, t6, t2
   13260: 00e0c733     	xor	a4, ra, a4
   13264: 01064633     	xor	a2, a2, a6
   13268: 0068c833     	xor	a6, a7, t1
   1326c: 07c12c23     	sw	t3, 0x78(sp)
   13270: 00ae5893     	srli	a7, t3, 0xa
   13274: 01154533     	xor	a0, a0, a7
   13278: 00af5893     	srli	a7, t5, 0xa
   1327c: 0114c333     	xor	t1, s1, a7
   13280: 01d686b3     	add	a3, a3, t4
   13284: 013948b3     	xor	a7, s2, s3
   13288: 01acceb3     	xor	t4, s9, s10
   1328c: 003f5493     	srli	s1, t5, 0x3
   13290: 000f0c13     	mv	s8, t5
   13294: 01e12823     	sw	t5, 0x10(sp)
   13298: 00964633     	xor	a2, a2, s1
   1329c: 02c12a23     	sw	a2, 0x34(sp)
   132a0: 003e5613     	srli	a2, t3, 0x3
   132a4: 00c84633     	xor	a2, a6, a2
   132a8: 02c12e23     	sw	a2, 0x3c(sp)
   132ac: 0c812603     	lw	a2, 0xc8(sp)
   132b0: 04412803     	lw	a6, 0x44(sp)
   132b4: 00c80633     	add	a2, a6, a2
   132b8: 00860633     	add	a2, a2, s0
   132bc: 00a60f33     	add	t5, a2, a0
   132c0: 07c12503     	lw	a0, 0x7c(sp)
   132c4: 05412603     	lw	a2, 0x54(sp)
   132c8: 00a60533     	add	a0, a2, a0
   132cc: 06412403     	lw	s0, 0x64(sp)
   132d0: 00850533     	add	a0, a0, s0
   132d4: 00650333     	add	t1, a0, t1
   132d8: 0158c533     	xor	a0, a7, s5
   132dc: 9bdc0637     	lui	a2, 0x9bdc0
   132e0: 6a760613     	addi	a2, a2, 0x6a7
   132e4: 00c68633     	add	a2, a3, a2
   132e8: 01bec6b3     	xor	a3, t4, s11
   132ec: 00a609b3     	add	s3, a2, a0
   132f0: 00e68733     	add	a4, a3, a4
   132f4: 011f5513     	srli	a0, t5, 0x11
   132f8: 00ff1613     	slli	a2, t5, 0xf
   132fc: 00a66633     	or	a2, a2, a0
   13300: 013f5513     	srli	a0, t5, 0x13
   13304: 00df1693     	slli	a3, t5, 0xd
   13308: 00a6e6b3     	or	a3, a3, a0
   1330c: 01135513     	srli	a0, t1, 0x11
   13310: 00f31813     	slli	a6, t1, 0xf
   13314: 00a86eb3     	or	t4, a6, a0
   13318: 01335513     	srli	a0, t1, 0x13
   1331c: 00d31493     	slli	s1, t1, 0xd
   13320: 00a4e4b3     	or	s1, s1, a0
   13324: 01698533     	add	a0, s3, s6
   13328: 013709b3     	add	s3, a4, s3
   1332c: 00735713     	srli	a4, t1, 0x7
   13330: 01931813     	slli	a6, t1, 0x19
   13334: 00e86733     	or	a4, a6, a4
   13338: 01235813     	srli	a6, t1, 0x12
   1333c: 00e31893     	slli	a7, t1, 0xe
   13340: 0108e833     	or	a6, a7, a6
   13344: 00d64633     	xor	a2, a2, a3
   13348: 009ec6b3     	xor	a3, t4, s1
   1334c: 01074733     	xor	a4, a4, a6
   13350: 00af5813     	srli	a6, t5, 0xa
   13354: 05e12023     	sw	t5, 0x40(sp)
   13358: 01064ab3     	xor	s5, a2, a6
   1335c: 00a35613     	srli	a2, t1, 0xa
   13360: 00c6c633     	xor	a2, a3, a2
   13364: 0cc12d03     	lw	s10, 0xcc(sp)
   13368: 00fd0d33     	add	s10, s10, a5
   1336c: 00fbc6b3     	xor	a3, s7, a5
   13370: 00d576b3     	and	a3, a0, a3
   13374: 00f6c6b3     	xor	a3, a3, a5
   13378: 00655793     	srli	a5, a0, 0x6
   1337c: 01a51813     	slli	a6, a0, 0x1a
   13380: 00f867b3     	or	a5, a6, a5
   13384: 00b55813     	srli	a6, a0, 0xb
   13388: 01551893     	slli	a7, a0, 0x15
   1338c: 0108e8b3     	or	a7, a7, a6
   13390: 01955813     	srli	a6, a0, 0x19
   13394: 00751e93     	slli	t4, a0, 0x7
   13398: 010ee4b3     	or	s1, t4, a6
   1339c: 0029d813     	srli	a6, s3, 0x2
   133a0: 01e99e93     	slli	t4, s3, 0x1e
   133a4: 010ee933     	or	s2, t4, a6
   133a8: 00d9d813     	srli	a6, s3, 0xd
   133ac: 01399e93     	slli	t4, s3, 0x13
   133b0: 010eecb3     	or	s9, t4, a6
   133b4: 0169d813     	srli	a6, s3, 0x16
   133b8: 00a99e93     	slli	t4, s3, 0xa
   133bc: 010eedb3     	or	s11, t4, a6
   133c0: 01fa4833     	xor	a6, s4, t6
   133c4: 0109f833     	and	a6, s3, a6
   133c8: 01fa7eb3     	and	t4, s4, t6
   133cc: 01d840b3     	xor	ra, a6, t4
   133d0: 00335813     	srli	a6, t1, 0x3
   133d4: 04612223     	sw	t1, 0x44(sp)
   133d8: 01074733     	xor	a4, a4, a6
   133dc: 00e12a23     	sw	a4, 0x14(sp)
   133e0: 05012703     	lw	a4, 0x50(sp)
   133e4: 0b012803     	lw	a6, 0xb0(sp)
   133e8: 01070733     	add	a4, a4, a6
   133ec: 00570733     	add	a4, a4, t0
   133f0: 01570ab3     	add	s5, a4, s5
   133f4: 08012703     	lw	a4, 0x80(sp)
   133f8: 05c12803     	lw	a6, 0x5c(sp)
   133fc: 00e80733     	add	a4, a6, a4
   13400: 01c12b03     	lw	s6, 0x1c(sp)
   13404: 01670733     	add	a4, a4, s6
   13408: 00c70eb3     	add	t4, a4, a2
   1340c: 00d585b3     	add	a1, a1, a3
   13410: 0117c633     	xor	a2, a5, a7
   13414: 019946b3     	xor	a3, s2, s9
   13418: 00964633     	xor	a2, a2, s1
   1341c: c19bf737     	lui	a4, 0xc19bf
   13420: 17470713     	addi	a4, a4, 0x174
   13424: 00e585b3     	add	a1, a1, a4
   13428: 01b6c6b3     	xor	a3, a3, s11
   1342c: 011ad713     	srli	a4, s5, 0x11
   13430: 00fa9793     	slli	a5, s5, 0xf
   13434: 00e7e733     	or	a4, a5, a4
   13438: 013ad793     	srli	a5, s5, 0x13
   1343c: 00da9893     	slli	a7, s5, 0xd
   13440: 0d512c23     	sw	s5, 0xd8(sp)
   13444: 00f8e7b3     	or	a5, a7, a5
   13448: 011ed893     	srli	a7, t4, 0x11
   1344c: 00fe9493     	slli	s1, t4, 0xf
   13450: 0114e8b3     	or	a7, s1, a7
   13454: 013ed493     	srli	s1, t4, 0x13
   13458: 00de9913     	slli	s2, t4, 0xd
   1345c: 009964b3     	or	s1, s2, s1
   13460: 00c585b3     	add	a1, a1, a2
   13464: 00168633     	add	a2, a3, ra
   13468: 00f74733     	xor	a4, a4, a5
   1346c: 0098c6b3     	xor	a3, a7, s1
   13470: 007583b3     	add	t2, a1, t2
   13474: 00b602b3     	add	t0, a2, a1
   13478: 00aad593     	srli	a1, s5, 0xa
   1347c: 00b745b3     	xor	a1, a4, a1
   13480: 00aed713     	srli	a4, t4, 0xa
   13484: 05d12823     	sw	t4, 0x50(sp)
   13488: 00e6c6b3     	xor	a3, a3, a4
   1348c: 04812703     	lw	a4, 0x48(sp)
   13490: 0bc12603     	lw	a2, 0xbc(sp)
   13494: 00c70733     	add	a4, a4, a2
   13498: 01870733     	add	a4, a4, s8
   1349c: 00b70833     	add	a6, a4, a1
   134a0: 05812583     	lw	a1, 0x58(sp)
   134a4: 07412603     	lw	a2, 0x74(sp)
   134a8: 00c585b3     	add	a1, a1, a2
   134ac: 01812c03     	lw	s8, 0x18(sp)
   134b0: 018585b3     	add	a1, a1, s8
   134b4: 00d58e33     	add	t3, a1, a3
   134b8: 0e412583     	lw	a1, 0xe4(sp)
   134bc: 017585b3     	add	a1, a1, s7
   134c0: 017546b3     	xor	a3, a0, s7
   134c4: 00d3f6b3     	and	a3, t2, a3
   134c8: 0176c6b3     	xor	a3, a3, s7
   134cc: 0063d713     	srli	a4, t2, 0x6
   134d0: 01a39793     	slli	a5, t2, 0x1a
   134d4: 00e7e733     	or	a4, a5, a4
   134d8: 00b3d793     	srli	a5, t2, 0xb
   134dc: 01539893     	slli	a7, t2, 0x15
   134e0: 00f8e7b3     	or	a5, a7, a5
   134e4: 0193d893     	srli	a7, t2, 0x19
   134e8: 00739493     	slli	s1, t2, 0x7
   134ec: 0114e8b3     	or	a7, s1, a7
   134f0: 0022d493     	srli	s1, t0, 0x2
   134f4: 01e29b93     	slli	s7, t0, 0x1e
   134f8: 009be4b3     	or	s1, s7, s1
   134fc: 00d2db93     	srli	s7, t0, 0xd
   13500: 01329d93     	slli	s11, t0, 0x13
   13504: 017debb3     	or	s7, s11, s7
   13508: 0162dd93     	srli	s11, t0, 0x16
   1350c: 00a29093     	slli	ra, t0, 0xa
   13510: 01b0edb3     	or	s11, ra, s11
   13514: 0149c0b3     	xor	ra, s3, s4
   13518: 0012f0b3     	and	ra, t0, ra
   1351c: 0149fcb3     	and	s9, s3, s4
   13520: 0190ccb3     	xor	s9, ra, s9
   13524: 00dd06b3     	add	a3, s10, a3
   13528: 00f74733     	xor	a4, a4, a5
   1352c: 0174c7b3     	xor	a5, s1, s7
   13530: 011e5493     	srli	s1, t3, 0x11
   13534: 00fe1b93     	slli	s7, t3, 0xf
   13538: 009be4b3     	or	s1, s7, s1
   1353c: 013e5b93     	srli	s7, t3, 0x13
   13540: 00de1d13     	slli	s10, t3, 0xd
   13544: 017d6bb3     	or	s7, s10, s7
   13548: 01174733     	xor	a4, a4, a7
   1354c: e49b78b7     	lui	a7, 0xe49b7
   13550: 9c188893     	addi	a7, a7, -0x63f
   13554: 011686b3     	add	a3, a3, a7
   13558: 01b7c7b3     	xor	a5, a5, s11
   1355c: 01185893     	srli	a7, a6, 0x11
   13560: 00f81d13     	slli	s10, a6, 0xf
   13564: 011d68b3     	or	a7, s10, a7
   13568: 01385d13     	srli	s10, a6, 0x13
   1356c: 00d81d93     	slli	s11, a6, 0xd
   13570: 0d012623     	sw	a6, 0xcc(sp)
   13574: 01aded33     	or	s10, s11, s10
   13578: 0174c4b3     	xor	s1, s1, s7
   1357c: 00e686b3     	add	a3, a3, a4
   13580: 019787b3     	add	a5, a5, s9
   13584: 01a8c733     	xor	a4, a7, s10
   13588: 00ae5893     	srli	a7, t3, 0xa
   1358c: 05c12a23     	sw	t3, 0x54(sp)
   13590: 0114cbb3     	xor	s7, s1, a7
   13594: 01f68fb3     	add	t6, a3, t6
   13598: 00d786b3     	add	a3, a5, a3
   1359c: 00a85793     	srli	a5, a6, 0xa
   135a0: 00f74733     	xor	a4, a4, a5
   135a4: 04c12783     	lw	a5, 0x4c(sp)
   135a8: 07012603     	lw	a2, 0x70(sp)
   135ac: 00c787b3     	add	a5, a5, a2
   135b0: 07812603     	lw	a2, 0x78(sp)
   135b4: 00c787b3     	add	a5, a5, a2
   135b8: 01778833     	add	a6, a5, s7
   135bc: 0b812783     	lw	a5, 0xb8(sp)
   135c0: 02412603     	lw	a2, 0x24(sp)
   135c4: 00f607b3     	add	a5, a2, a5
   135c8: 006787b3     	add	a5, a5, t1
   135cc: 00e78633     	add	a2, a5, a4
   135d0: 0dc12d83     	lw	s11, 0xdc(sp)
   135d4: 00ad8db3     	add	s11, s11, a0
   135d8: 00a3c733     	xor	a4, t2, a0
   135dc: 00eff733     	and	a4, t6, a4
   135e0: 00a74533     	xor	a0, a4, a0
   135e4: 006fd713     	srli	a4, t6, 0x6
   135e8: 01af9793     	slli	a5, t6, 0x1a
   135ec: 00e7e733     	or	a4, a5, a4
   135f0: 00bfd793     	srli	a5, t6, 0xb
   135f4: 015f9893     	slli	a7, t6, 0x15
   135f8: 00f8e7b3     	or	a5, a7, a5
   135fc: 019fd893     	srli	a7, t6, 0x19
   13600: 007f9493     	slli	s1, t6, 0x7
   13604: 0114e8b3     	or	a7, s1, a7
   13608: 0026d493     	srli	s1, a3, 0x2
   1360c: 01e69c93     	slli	s9, a3, 0x1e
   13610: 009ce4b3     	or	s1, s9, s1
   13614: 00d6dc93     	srli	s9, a3, 0xd
   13618: 01369d13     	slli	s10, a3, 0x13
   1361c: 019d6cb3     	or	s9, s10, s9
   13620: 0166dd13     	srli	s10, a3, 0x16
   13624: 00a69093     	slli	ra, a3, 0xa
   13628: 01a0ed33     	or	s10, ra, s10
   1362c: 0132c0b3     	xor	ra, t0, s3
   13630: 0016f0b3     	and	ra, a3, ra
   13634: 0132fbb3     	and	s7, t0, s3
   13638: 0170cbb3     	xor	s7, ra, s7
   1363c: 00080313     	mv	t1, a6
   13640: 01185093     	srli	ra, a6, 0x11
   13644: 00f81a93     	slli	s5, a6, 0xf
   13648: 001aeab3     	or	s5, s5, ra
   1364c: 01385093     	srli	ra, a6, 0x13
   13650: 00d81913     	slli	s2, a6, 0xd
   13654: 00196933     	or	s2, s2, ra
   13658: 00a58533     	add	a0, a1, a0
   1365c: 00f74733     	xor	a4, a4, a5
   13660: 0194c5b3     	xor	a1, s1, s9
   13664: 01165793     	srli	a5, a2, 0x11
   13668: 00f61493     	slli	s1, a2, 0xf
   1366c: 00f4e7b3     	or	a5, s1, a5
   13670: 01365493     	srli	s1, a2, 0x13
   13674: 00d61c93     	slli	s9, a2, 0xd
   13678: 0ec12223     	sw	a2, 0xe4(sp)
   1367c: 009ce4b3     	or	s1, s9, s1
   13680: 012ac933     	xor	s2, s5, s2
   13684: 01174733     	xor	a4, a4, a7
   13688: efbe48b7     	lui	a7, 0xefbe4
   1368c: 78688893     	addi	a7, a7, 0x786
   13690: 01150533     	add	a0, a0, a7
   13694: 01a5c5b3     	xor	a1, a1, s10
   13698: 0097c7b3     	xor	a5, a5, s1
   1369c: 00a85893     	srli	a7, a6, 0xa
   136a0: 05012c23     	sw	a6, 0x58(sp)
   136a4: 01194d33     	xor	s10, s2, a7
   136a8: 00e50533     	add	a0, a0, a4
   136ac: 017585b3     	add	a1, a1, s7
   136b0: 00a65713     	srli	a4, a2, 0xa
   136b4: 00e7c733     	xor	a4, a5, a4
   136b8: 06c12783     	lw	a5, 0x6c(sp)
   136bc: 02812603     	lw	a2, 0x28(sp)
   136c0: 00f607b3     	add	a5, a2, a5
   136c4: 01e787b3     	add	a5, a5, t5
   136c8: 01a78d33     	add	s10, a5, s10
   136cc: 01450a33     	add	s4, a0, s4
   136d0: 00a58833     	add	a6, a1, a0
   136d4: 02c12503     	lw	a0, 0x2c(sp)
   136d8: 00850533     	add	a0, a0, s0
   136dc: 01d50533     	add	a0, a0, t4
   136e0: 00e505b3     	add	a1, a0, a4
   136e4: 011d5513     	srli	a0, s10, 0x11
   136e8: 00fd1713     	slli	a4, s10, 0xf
   136ec: 00a76533     	or	a0, a4, a0
   136f0: 013d5713     	srli	a4, s10, 0x13
   136f4: 00dd1793     	slli	a5, s10, 0xd
   136f8: 000d0413     	mv	s0, s10
   136fc: 00e7e733     	or	a4, a5, a4
   13700: 0c412083     	lw	ra, 0xc4(sp)
   13704: 007080b3     	add	ra, ra, t2
   13708: 007fc7b3     	xor	a5, t6, t2
   1370c: 00fa77b3     	and	a5, s4, a5
   13710: 0077c7b3     	xor	a5, a5, t2
   13714: 006a5893     	srli	a7, s4, 0x6
   13718: 01aa1393     	slli	t2, s4, 0x1a
   1371c: 0113e8b3     	or	a7, t2, a7
   13720: 00ba5393     	srli	t2, s4, 0xb
   13724: 015a1493     	slli	s1, s4, 0x15
   13728: 0074e3b3     	or	t2, s1, t2
   1372c: 019a5493     	srli	s1, s4, 0x19
   13730: 007a1913     	slli	s2, s4, 0x7
   13734: 009964b3     	or	s1, s2, s1
   13738: 00285913     	srli	s2, a6, 0x2
   1373c: 01e81a93     	slli	s5, a6, 0x1e
   13740: 012ae933     	or	s2, s5, s2
   13744: 00d85a93     	srli	s5, a6, 0xd
   13748: 01381b93     	slli	s7, a6, 0x13
   1374c: 015beab3     	or	s5, s7, s5
   13750: 01685b93     	srli	s7, a6, 0x16
   13754: 00a81c93     	slli	s9, a6, 0xa
   13758: 017cebb3     	or	s7, s9, s7
   1375c: 0056ccb3     	xor	s9, a3, t0
   13760: 01987cb3     	and	s9, a6, s9
   13764: 0056f633     	and	a2, a3, t0
   13768: 00ccc633     	xor	a2, s9, a2
   1376c: 0115dc93     	srli	s9, a1, 0x11
   13770: 00f59d13     	slli	s10, a1, 0xf
   13774: 019d6cb3     	or	s9, s10, s9
   13778: 0135dd13     	srli	s10, a1, 0x13
   1377c: 00d59e93     	slli	t4, a1, 0xd
   13780: 04b12e23     	sw	a1, 0x5c(sp)
   13784: 01aeeeb3     	or	t4, t4, s10
   13788: 00e54533     	xor	a0, a0, a4
   1378c: 00fd87b3     	add	a5, s11, a5
   13790: 0078c733     	xor	a4, a7, t2
   13794: 015948b3     	xor	a7, s2, s5
   13798: 01dcc3b3     	xor	t2, s9, t4
   1379c: 00a45e93     	srli	t4, s0, 0xa
   137a0: 01d54533     	xor	a0, a0, t4
   137a4: 00974733     	xor	a4, a4, s1
   137a8: 0fc1aeb7     	lui	t4, 0xfc1a
   137ac: dc6e8e93     	addi	t4, t4, -0x23a
   137b0: 01d787b3     	add	a5, a5, t4
   137b4: 0178c8b3     	xor	a7, a7, s7
   137b8: 00a5de93     	srli	t4, a1, 0xa
   137bc: 01d3cdb3     	xor	s11, t2, t4
   137c0: 0ac12383     	lw	t2, 0xac(sp)
   137c4: 03012583     	lw	a1, 0x30(sp)
   137c8: 007583b3     	add	t2, a1, t2
   137cc: 0d812583     	lw	a1, 0xd8(sp)
   137d0: 00b383b3     	add	t2, t2, a1
   137d4: 00a383b3     	add	t2, t2, a0
   137d8: 00e78533     	add	a0, a5, a4
   137dc: 00c88633     	add	a2, a7, a2
   137e0: 02012703     	lw	a4, 0x20(sp)
   137e4: 01670733     	add	a4, a4, s6
   137e8: 01c70733     	add	a4, a4, t3
   137ec: 01b70db3     	add	s11, a4, s11
   137f0: 013509b3     	add	s3, a0, s3
   137f4: 00a60533     	add	a0, a2, a0
   137f8: 0113d613     	srli	a2, t2, 0x11
   137fc: 00f39713     	slli	a4, t2, 0xf
   13800: 00c76633     	or	a2, a4, a2
   13804: 0133d713     	srli	a4, t2, 0x13
   13808: 00d39793     	slli	a5, t2, 0xd
   1380c: 00e7e733     	or	a4, a5, a4
   13810: 011dd793     	srli	a5, s11, 0x11
   13814: 00fd9893     	slli	a7, s11, 0xf
   13818: 00f8e8b3     	or	a7, a7, a5
   1381c: 013dd793     	srli	a5, s11, 0x13
   13820: 00dd9e93     	slli	t4, s11, 0xd
   13824: 00feeeb3     	or	t4, t4, a5
   13828: 00e64633     	xor	a2, a2, a4
   1382c: 08812783     	lw	a5, 0x88(sp)
   13830: 01f787b3     	add	a5, a5, t6
   13834: 01fa4733     	xor	a4, s4, t6
   13838: 00e9f733     	and	a4, s3, a4
   1383c: 01f74733     	xor	a4, a4, t6
   13840: 0069df93     	srli	t6, s3, 0x6
   13844: 01a99493     	slli	s1, s3, 0x1a
   13848: 01f4efb3     	or	t6, s1, t6
   1384c: 00b9d493     	srli	s1, s3, 0xb
   13850: 01599913     	slli	s2, s3, 0x15
   13854: 009964b3     	or	s1, s2, s1
   13858: 0199d913     	srli	s2, s3, 0x19
   1385c: 00799a93     	slli	s5, s3, 0x7
   13860: 012ae933     	or	s2, s5, s2
   13864: 00255a93     	srli	s5, a0, 0x2
   13868: 01e51b93     	slli	s7, a0, 0x1e
   1386c: 015beab3     	or	s5, s7, s5
   13870: 00d55b93     	srli	s7, a0, 0xd
   13874: 01351c93     	slli	s9, a0, 0x13
   13878: 017cebb3     	or	s7, s9, s7
   1387c: 01655c93     	srli	s9, a0, 0x16
   13880: 00a51d13     	slli	s10, a0, 0xa
   13884: 019d6cb3     	or	s9, s10, s9
   13888: 00d84d33     	xor	s10, a6, a3
   1388c: 01a57d33     	and	s10, a0, s10
   13890: 00d875b3     	and	a1, a6, a3
   13894: 00bd45b3     	xor	a1, s10, a1
   13898: 01d8c8b3     	xor	a7, a7, t4
   1389c: 00a3de93     	srli	t4, t2, 0xa
   138a0: 01d64633     	xor	a2, a2, t4
   138a4: 00e08733     	add	a4, ra, a4
   138a8: 009fceb3     	xor	t4, t6, s1
   138ac: 017ac4b3     	xor	s1, s5, s7
   138b0: 00addf93     	srli	t6, s11, 0xa
   138b4: 01f8c0b3     	xor	ra, a7, t6
   138b8: 0c012883     	lw	a7, 0xc0(sp)
   138bc: 03812e03     	lw	t3, 0x38(sp)
   138c0: 011e08b3     	add	a7, t3, a7
   138c4: 0cc12e03     	lw	t3, 0xcc(sp)
   138c8: 01c888b3     	add	a7, a7, t3
   138cc: 00c88fb3     	add	t6, a7, a2
   138d0: 012ec633     	xor	a2, t4, s2
   138d4: 240ca8b7     	lui	a7, 0x240ca
   138d8: 1cc88893     	addi	a7, a7, 0x1cc
   138dc: 01170733     	add	a4, a4, a7
   138e0: 0194c8b3     	xor	a7, s1, s9
   138e4: 03412e83     	lw	t4, 0x34(sp)
   138e8: 018e8eb3     	add	t4, t4, s8
   138ec: 006e8eb3     	add	t4, t4, t1
   138f0: 001e8f33     	add	t5, t4, ra
   138f4: 00c70733     	add	a4, a4, a2
   138f8: 00b885b3     	add	a1, a7, a1
   138fc: 00570633     	add	a2, a4, t0
   13900: 00e58733     	add	a4, a1, a4
   13904: 011fd593     	srli	a1, t6, 0x11
   13908: 00ff9893     	slli	a7, t6, 0xf
   1390c: 00b8e5b3     	or	a1, a7, a1
   13910: 013fd893     	srli	a7, t6, 0x13
   13914: 00df9293     	slli	t0, t6, 0xd
   13918: 0112e8b3     	or	a7, t0, a7
   1391c: 011f5293     	srli	t0, t5, 0x11
   13920: 00ff1e93     	slli	t4, t5, 0xf
   13924: 005ee2b3     	or	t0, t4, t0
   13928: 013f5e93     	srli	t4, t5, 0x13
   1392c: 00df1493     	slli	s1, t5, 0xd
   13930: 0de12223     	sw	t5, 0xc4(sp)
   13934: 01d4eeb3     	or	t4, s1, t4
   13938: 0115c5b3     	xor	a1, a1, a7
   1393c: 01d2c8b3     	xor	a7, t0, t4
   13940: 0d412483     	lw	s1, 0xd4(sp)
   13944: 014484b3     	add	s1, s1, s4
   13948: 0149c2b3     	xor	t0, s3, s4
   1394c: 005672b3     	and	t0, a2, t0
   13950: 0142c2b3     	xor	t0, t0, s4
   13954: 00665e93     	srli	t4, a2, 0x6
   13958: 01a61913     	slli	s2, a2, 0x1a
   1395c: 01d96eb3     	or	t4, s2, t4
   13960: 00b65913     	srli	s2, a2, 0xb
   13964: 01561a13     	slli	s4, a2, 0x15
   13968: 012a6933     	or	s2, s4, s2
   1396c: 01965a13     	srli	s4, a2, 0x19
   13970: 00761a93     	slli	s5, a2, 0x7
   13974: 014aea33     	or	s4, s5, s4
   13978: 00275a93     	srli	s5, a4, 0x2
   1397c: 01e71b93     	slli	s7, a4, 0x1e
   13980: 015beab3     	or	s5, s7, s5
   13984: 00d75b93     	srli	s7, a4, 0xd
   13988: 01371c93     	slli	s9, a4, 0x13
   1398c: 017cebb3     	or	s7, s9, s7
   13990: 01675c93     	srli	s9, a4, 0x16
   13994: 00a71d13     	slli	s10, a4, 0xa
   13998: 019d6cb3     	or	s9, s10, s9
   1399c: 01054d33     	xor	s10, a0, a6
   139a0: 01a77d33     	and	s10, a4, s10
   139a4: 010570b3     	and	ra, a0, a6
   139a8: 001d4d33     	xor	s10, s10, ra
   139ac: 00afd093     	srli	ra, t6, 0xa
   139b0: 0015c5b3     	xor	a1, a1, ra
   139b4: 00af5093     	srli	ra, t5, 0xa
   139b8: 0018c8b3     	xor	a7, a7, ra
   139bc: 005787b3     	add	a5, a5, t0
   139c0: 012ec2b3     	xor	t0, t4, s2
   139c4: 017aceb3     	xor	t4, s5, s7
   139c8: 01012e03     	lw	t3, 0x10(sp)
   139cc: 03c12903     	lw	s2, 0x3c(sp)
   139d0: 012e0933     	add	s2, t3, s2
   139d4: 0e412f03     	lw	t5, 0xe4(sp)
   139d8: 01e90933     	add	s2, s2, t5
   139dc: 00b905b3     	add	a1, s2, a1
   139e0: 0cb12e23     	sw	a1, 0xdc(sp)
   139e4: 07812303     	lw	t1, 0x78(sp)
   139e8: 01412583     	lw	a1, 0x14(sp)
   139ec: 00b305b3     	add	a1, t1, a1
   139f0: 008585b3     	add	a1, a1, s0
   139f4: 00040093     	mv	ra, s0
   139f8: 011585b3     	add	a1, a1, a7
   139fc: 0cb12a23     	sw	a1, 0xd4(sp)
   13a00: 0142c5b3     	xor	a1, t0, s4
   13a04: 2de938b7     	lui	a7, 0x2de93
   13a08: c6f88893     	addi	a7, a7, -0x391
   13a0c: 011787b3     	add	a5, a5, a7
   13a10: 019ec8b3     	xor	a7, t4, s9
   13a14: 00b785b3     	add	a1, a5, a1
   13a18: 01a888b3     	add	a7, a7, s10
   13a1c: 00d587b3     	add	a5, a1, a3
   13a20: 00b886b3     	add	a3, a7, a1
   13a24: 09412883     	lw	a7, 0x94(sp)
   13a28: 013888b3     	add	a7, a7, s3
   13a2c: 013645b3     	xor	a1, a2, s3
   13a30: 00b7f5b3     	and	a1, a5, a1
   13a34: 0135c5b3     	xor	a1, a1, s3
   13a38: 0067d293     	srli	t0, a5, 0x6
   13a3c: 01a79e93     	slli	t4, a5, 0x1a
   13a40: 005ee2b3     	or	t0, t4, t0
   13a44: 00b7de93     	srli	t4, a5, 0xb
   13a48: 01579913     	slli	s2, a5, 0x15
   13a4c: 01d96eb3     	or	t4, s2, t4
   13a50: 0197d913     	srli	s2, a5, 0x19
   13a54: 00779993     	slli	s3, a5, 0x7
   13a58: 0129e933     	or	s2, s3, s2
   13a5c: 0026d993     	srli	s3, a3, 0x2
   13a60: 01e69a13     	slli	s4, a3, 0x1e
   13a64: 013a69b3     	or	s3, s4, s3
   13a68: 00d6da13     	srli	s4, a3, 0xd
   13a6c: 01369a93     	slli	s5, a3, 0x13
   13a70: 014aea33     	or	s4, s5, s4
   13a74: 0166da93     	srli	s5, a3, 0x16
   13a78: 00a69b93     	slli	s7, a3, 0xa
   13a7c: 015beab3     	or	s5, s7, s5
   13a80: 00a74bb3     	xor	s7, a4, a0
   13a84: 0176fbb3     	and	s7, a3, s7
   13a88: 00a77cb3     	and	s9, a4, a0
   13a8c: 019bcbb3     	xor	s7, s7, s9
   13a90: 00b485b3     	add	a1, s1, a1
   13a94: 01d2c2b3     	xor	t0, t0, t4
   13a98: 0149ceb3     	xor	t4, s3, s4
   13a9c: 0122c2b3     	xor	t0, t0, s2
   13aa0: 4a7484b7     	lui	s1, 0x4a748
   13aa4: 4aa48493     	addi	s1, s1, 0x4aa
   13aa8: 009585b3     	add	a1, a1, s1
   13aac: 015eceb3     	xor	t4, t4, s5
   13ab0: 005585b3     	add	a1, a1, t0
   13ab4: 017e8eb3     	add	t4, t4, s7
   13ab8: 010584b3     	add	s1, a1, a6
   13abc: 00be85b3     	add	a1, t4, a1
   13ac0: 0ec12a03     	lw	s4, 0xec(sp)
   13ac4: 00ca0a33     	add	s4, s4, a2
   13ac8: 00c7c833     	xor	a6, a5, a2
   13acc: 0104f833     	and	a6, s1, a6
   13ad0: 00c84633     	xor	a2, a6, a2
   13ad4: 0064d813     	srli	a6, s1, 0x6
   13ad8: 01a49293     	slli	t0, s1, 0x1a
   13adc: 0102e833     	or	a6, t0, a6
   13ae0: 00b4d293     	srli	t0, s1, 0xb
   13ae4: 01549e93     	slli	t4, s1, 0x15
   13ae8: 005ee2b3     	or	t0, t4, t0
   13aec: 0194de93     	srli	t4, s1, 0x19
   13af0: 00749913     	slli	s2, s1, 0x7
   13af4: 01d96eb3     	or	t4, s2, t4
   13af8: 0025d913     	srli	s2, a1, 0x2
   13afc: 01e59993     	slli	s3, a1, 0x1e
   13b00: 0129e933     	or	s2, s3, s2
   13b04: 00d5d993     	srli	s3, a1, 0xd
   13b08: 01359a93     	slli	s5, a1, 0x13
   13b0c: 013ae9b3     	or	s3, s5, s3
   13b10: 0165da93     	srli	s5, a1, 0x16
   13b14: 00a59b93     	slli	s7, a1, 0xa
   13b18: 015beab3     	or	s5, s7, s5
   13b1c: 00e6cbb3     	xor	s7, a3, a4
   13b20: 0175fbb3     	and	s7, a1, s7
   13b24: 00e6fcb3     	and	s9, a3, a4
   13b28: 019bcbb3     	xor	s7, s7, s9
   13b2c: 00c88633     	add	a2, a7, a2
   13b30: 00584833     	xor	a6, a6, t0
   13b34: 013948b3     	xor	a7, s2, s3
   13b38: 01d84833     	xor	a6, a6, t4
   13b3c: 5cb0b2b7     	lui	t0, 0x5cb0b
   13b40: 9dc28293     	addi	t0, t0, -0x624
   13b44: 00560633     	add	a2, a2, t0
   13b48: 0158c8b3     	xor	a7, a7, s5
   13b4c: 01060633     	add	a2, a2, a6
   13b50: 017888b3     	add	a7, a7, s7
   13b54: 00a609b3     	add	s3, a2, a0
   13b58: 00c88533     	add	a0, a7, a2
   13b5c: 09012803     	lw	a6, 0x90(sp)
   13b60: 00f80833     	add	a6, a6, a5
   13b64: 00f4c633     	xor	a2, s1, a5
   13b68: 00c9f633     	and	a2, s3, a2
   13b6c: 00f64633     	xor	a2, a2, a5
   13b70: 0069d793     	srli	a5, s3, 0x6
   13b74: 01a99893     	slli	a7, s3, 0x1a
   13b78: 00f8e7b3     	or	a5, a7, a5
   13b7c: 00b9d893     	srli	a7, s3, 0xb
   13b80: 01599293     	slli	t0, s3, 0x15
   13b84: 0112e8b3     	or	a7, t0, a7
   13b88: 0199d293     	srli	t0, s3, 0x19
   13b8c: 00799e93     	slli	t4, s3, 0x7
   13b90: 005ee2b3     	or	t0, t4, t0
   13b94: 00255e93     	srli	t4, a0, 0x2
   13b98: 01e51913     	slli	s2, a0, 0x1e
   13b9c: 01d96eb3     	or	t4, s2, t4
   13ba0: 00d55913     	srli	s2, a0, 0xd
   13ba4: 01351a93     	slli	s5, a0, 0x13
   13ba8: 012ae933     	or	s2, s5, s2
   13bac: 01655a93     	srli	s5, a0, 0x16
   13bb0: 00a51b93     	slli	s7, a0, 0xa
   13bb4: 015beab3     	or	s5, s7, s5
   13bb8: 00d5cbb3     	xor	s7, a1, a3
   13bbc: 01757bb3     	and	s7, a0, s7
   13bc0: 00d5fcb3     	and	s9, a1, a3
   13bc4: 019bcbb3     	xor	s7, s7, s9
   13bc8: 00ca0633     	add	a2, s4, a2
   13bcc: 0117c7b3     	xor	a5, a5, a7
   13bd0: 012ec8b3     	xor	a7, t4, s2
   13bd4: 0057c7b3     	xor	a5, a5, t0
   13bd8: 76f992b7     	lui	t0, 0x76f99
   13bdc: 8da28293     	addi	t0, t0, -0x726
   13be0: 00560633     	add	a2, a2, t0
   13be4: 0158c8b3     	xor	a7, a7, s5
   13be8: 00f60633     	add	a2, a2, a5
   13bec: 017888b3     	add	a7, a7, s7
   13bf0: 00e60733     	add	a4, a2, a4
   13bf4: 00c88633     	add	a2, a7, a2
   13bf8: 0d012a03     	lw	s4, 0xd0(sp)
   13bfc: 009a0a33     	add	s4, s4, s1
   13c00: 0099c7b3     	xor	a5, s3, s1
   13c04: 00f777b3     	and	a5, a4, a5
   13c08: 0097c7b3     	xor	a5, a5, s1
   13c0c: 00675893     	srli	a7, a4, 0x6
   13c10: 01a71293     	slli	t0, a4, 0x1a
   13c14: 0112e8b3     	or	a7, t0, a7
   13c18: 00b75293     	srli	t0, a4, 0xb
   13c1c: 01571e93     	slli	t4, a4, 0x15
   13c20: 005ee2b3     	or	t0, t4, t0
   13c24: 01975e93     	srli	t4, a4, 0x19
   13c28: 00771493     	slli	s1, a4, 0x7
   13c2c: 01d4eeb3     	or	t4, s1, t4
   13c30: 00265493     	srli	s1, a2, 0x2
   13c34: 01e61913     	slli	s2, a2, 0x1e
   13c38: 009964b3     	or	s1, s2, s1
   13c3c: 00d65913     	srli	s2, a2, 0xd
   13c40: 01361a93     	slli	s5, a2, 0x13
   13c44: 012ae933     	or	s2, s5, s2
   13c48: 01665a93     	srli	s5, a2, 0x16
   13c4c: 00a61b93     	slli	s7, a2, 0xa
   13c50: 015beab3     	or	s5, s7, s5
   13c54: 00b54bb3     	xor	s7, a0, a1
   13c58: 01767bb3     	and	s7, a2, s7
   13c5c: 00b57cb3     	and	s9, a0, a1
   13c60: 019bcbb3     	xor	s7, s7, s9
   13c64: 00f807b3     	add	a5, a6, a5
   13c68: 0058c833     	xor	a6, a7, t0
   13c6c: 0124c8b3     	xor	a7, s1, s2
   13c70: 01d84833     	xor	a6, a6, t4
   13c74: 983e52b7     	lui	t0, 0x983e5
   13c78: 15228293     	addi	t0, t0, 0x152
   13c7c: 005787b3     	add	a5, a5, t0
   13c80: 0158c8b3     	xor	a7, a7, s5
   13c84: 01078833     	add	a6, a5, a6
   13c88: 017888b3     	add	a7, a7, s7
   13c8c: 00d807b3     	add	a5, a6, a3
   13c90: 010886b3     	add	a3, a7, a6
   13c94: 08c12803     	lw	a6, 0x8c(sp)
   13c98: 01380833     	add	a6, a6, s3
   13c9c: 013748b3     	xor	a7, a4, s3
   13ca0: 0117f8b3     	and	a7, a5, a7
   13ca4: 0138c8b3     	xor	a7, a7, s3
   13ca8: 0067d293     	srli	t0, a5, 0x6
   13cac: 01a79e93     	slli	t4, a5, 0x1a
   13cb0: 005ee2b3     	or	t0, t4, t0
   13cb4: 00b7de93     	srli	t4, a5, 0xb
   13cb8: 01579493     	slli	s1, a5, 0x15
   13cbc: 01d4eeb3     	or	t4, s1, t4
   13cc0: 0197d493     	srli	s1, a5, 0x19
   13cc4: 00779913     	slli	s2, a5, 0x7
   13cc8: 009964b3     	or	s1, s2, s1
   13ccc: 0026d913     	srli	s2, a3, 0x2
   13cd0: 01e69993     	slli	s3, a3, 0x1e
   13cd4: 0129e933     	or	s2, s3, s2
   13cd8: 00d6d993     	srli	s3, a3, 0xd
   13cdc: 01369a93     	slli	s5, a3, 0x13
   13ce0: 013ae9b3     	or	s3, s5, s3
   13ce4: 0166da93     	srli	s5, a3, 0x16
   13ce8: 00a69b93     	slli	s7, a3, 0xa
   13cec: 015beab3     	or	s5, s7, s5
   13cf0: 00a64bb3     	xor	s7, a2, a0
   13cf4: 0176fbb3     	and	s7, a3, s7
   13cf8: 00a67cb3     	and	s9, a2, a0
   13cfc: 019bcbb3     	xor	s7, s7, s9
   13d00: 011a08b3     	add	a7, s4, a7
   13d04: 01d2c2b3     	xor	t0, t0, t4
   13d08: 01394eb3     	xor	t4, s2, s3
   13d0c: 0092c2b3     	xor	t0, t0, s1
   13d10: a831c4b7     	lui	s1, 0xa831c
   13d14: 66d48493     	addi	s1, s1, 0x66d
   13d18: 009888b3     	add	a7, a7, s1
   13d1c: 015eceb3     	xor	t4, t4, s5
   13d20: 005888b3     	add	a7, a7, t0
   13d24: 017e8eb3     	add	t4, t4, s7
   13d28: 00b884b3     	add	s1, a7, a1
   13d2c: 011e85b3     	add	a1, t4, a7
   13d30: 0b412983     	lw	s3, 0xb4(sp)
   13d34: 00e989b3     	add	s3, s3, a4
   13d38: 00e7c8b3     	xor	a7, a5, a4
   13d3c: 0114f8b3     	and	a7, s1, a7
   13d40: 00e8c733     	xor	a4, a7, a4
   13d44: 0064d893     	srli	a7, s1, 0x6
   13d48: 01a49293     	slli	t0, s1, 0x1a
   13d4c: 0112e8b3     	or	a7, t0, a7
   13d50: 00b4d293     	srli	t0, s1, 0xb
   13d54: 01549e93     	slli	t4, s1, 0x15
   13d58: 005ee2b3     	or	t0, t4, t0
   13d5c: 0194de93     	srli	t4, s1, 0x19
   13d60: 00749913     	slli	s2, s1, 0x7
   13d64: 01d96eb3     	or	t4, s2, t4
   13d68: 0025d913     	srli	s2, a1, 0x2
   13d6c: 01e59a13     	slli	s4, a1, 0x1e
   13d70: 012a6933     	or	s2, s4, s2
   13d74: 00d5da13     	srli	s4, a1, 0xd
   13d78: 01359a93     	slli	s5, a1, 0x13
   13d7c: 014aea33     	or	s4, s5, s4
   13d80: 0165da93     	srli	s5, a1, 0x16
   13d84: 00a59b93     	slli	s7, a1, 0xa
   13d88: 015beab3     	or	s5, s7, s5
   13d8c: 00c6cbb3     	xor	s7, a3, a2
   13d90: 0175fbb3     	and	s7, a1, s7
   13d94: 00c6fcb3     	and	s9, a3, a2
   13d98: 019bcbb3     	xor	s7, s7, s9
   13d9c: 00e80733     	add	a4, a6, a4
   13da0: 0058c833     	xor	a6, a7, t0
   13da4: 014948b3     	xor	a7, s2, s4
   13da8: 01d84833     	xor	a6, a6, t4
   13dac: b00322b7     	lui	t0, 0xb0032
   13db0: 7c828293     	addi	t0, t0, 0x7c8
   13db4: 00570733     	add	a4, a4, t0
   13db8: 0158c8b3     	xor	a7, a7, s5
   13dbc: 01070833     	add	a6, a4, a6
   13dc0: 017888b3     	add	a7, a7, s7
   13dc4: 00a80733     	add	a4, a6, a0
   13dc8: 01088533     	add	a0, a7, a6
   13dcc: 08412803     	lw	a6, 0x84(sp)
   13dd0: 00f80833     	add	a6, a6, a5
   13dd4: 00f4c8b3     	xor	a7, s1, a5
   13dd8: 011778b3     	and	a7, a4, a7
   13ddc: 00f8c7b3     	xor	a5, a7, a5
   13de0: 00675893     	srli	a7, a4, 0x6
   13de4: 01a71293     	slli	t0, a4, 0x1a
   13de8: 0112e8b3     	or	a7, t0, a7
   13dec: 00b75293     	srli	t0, a4, 0xb
   13df0: 01571e93     	slli	t4, a4, 0x15
   13df4: 005ee2b3     	or	t0, t4, t0
   13df8: 01975e93     	srli	t4, a4, 0x19
   13dfc: 00771913     	slli	s2, a4, 0x7
   13e00: 01d96eb3     	or	t4, s2, t4
   13e04: 00255913     	srli	s2, a0, 0x2
   13e08: 01e51a13     	slli	s4, a0, 0x1e
   13e0c: 012a6933     	or	s2, s4, s2
   13e10: 00d55a13     	srli	s4, a0, 0xd
   13e14: 01351a93     	slli	s5, a0, 0x13
   13e18: 014aea33     	or	s4, s5, s4
   13e1c: 01655a93     	srli	s5, a0, 0x16
   13e20: 00a51b93     	slli	s7, a0, 0xa
   13e24: 015beab3     	or	s5, s7, s5
   13e28: 00d5cbb3     	xor	s7, a1, a3
   13e2c: 01757bb3     	and	s7, a0, s7
   13e30: 00d5fcb3     	and	s9, a1, a3
   13e34: 019bcbb3     	xor	s7, s7, s9
   13e38: 00f987b3     	add	a5, s3, a5
   13e3c: 0058c8b3     	xor	a7, a7, t0
   13e40: 014942b3     	xor	t0, s2, s4
   13e44: 01d8c8b3     	xor	a7, a7, t4
   13e48: bf598eb7     	lui	t4, 0xbf598
   13e4c: fc7e8e93     	addi	t4, t4, -0x39
   13e50: 01d787b3     	add	a5, a5, t4
   13e54: 0152c2b3     	xor	t0, t0, s5
   13e58: 011788b3     	add	a7, a5, a7
   13e5c: 017282b3     	add	t0, t0, s7
   13e60: 00c887b3     	add	a5, a7, a2
   13e64: 01128633     	add	a2, t0, a7
   13e68: 06812983     	lw	s3, 0x68(sp)
   13e6c: 009989b3     	add	s3, s3, s1
   13e70: 009748b3     	xor	a7, a4, s1
   13e74: 0117f8b3     	and	a7, a5, a7
   13e78: 0098c8b3     	xor	a7, a7, s1
   13e7c: 0067d293     	srli	t0, a5, 0x6
   13e80: 01a79e93     	slli	t4, a5, 0x1a
   13e84: 005ee2b3     	or	t0, t4, t0
   13e88: 00b7de93     	srli	t4, a5, 0xb
   13e8c: 01579493     	slli	s1, a5, 0x15
   13e90: 01d4eeb3     	or	t4, s1, t4
   13e94: 0197d493     	srli	s1, a5, 0x19
   13e98: 00779913     	slli	s2, a5, 0x7
   13e9c: 009964b3     	or	s1, s2, s1
   13ea0: 00265913     	srli	s2, a2, 0x2
   13ea4: 01e61a13     	slli	s4, a2, 0x1e
   13ea8: 012a6933     	or	s2, s4, s2
   13eac: 00d65a13     	srli	s4, a2, 0xd
   13eb0: 01361a93     	slli	s5, a2, 0x13
   13eb4: 014aea33     	or	s4, s5, s4
   13eb8: 01665a93     	srli	s5, a2, 0x16
   13ebc: 00a61b93     	slli	s7, a2, 0xa
   13ec0: 015beab3     	or	s5, s7, s5
   13ec4: 00b54bb3     	xor	s7, a0, a1
   13ec8: 01767bb3     	and	s7, a2, s7
   13ecc: 00b57cb3     	and	s9, a0, a1
   13ed0: 019bcbb3     	xor	s7, s7, s9
   13ed4: 01180833     	add	a6, a6, a7
   13ed8: 01d2c8b3     	xor	a7, t0, t4
   13edc: 014942b3     	xor	t0, s2, s4
   13ee0: 0098c8b3     	xor	a7, a7, s1
   13ee4: c6e01eb7     	lui	t4, 0xc6e01
   13ee8: bf3e8e93     	addi	t4, t4, -0x40d
   13eec: 01d80833     	add	a6, a6, t4
   13ef0: 0152c2b3     	xor	t0, t0, s5
   13ef4: 01180833     	add	a6, a6, a7
   13ef8: 017282b3     	add	t0, t0, s7
   13efc: 00d804b3     	add	s1, a6, a3
   13f00: 010286b3     	add	a3, t0, a6
   13f04: 0e012803     	lw	a6, 0xe0(sp)
   13f08: 00e80833     	add	a6, a6, a4
   13f0c: 00e7c8b3     	xor	a7, a5, a4
   13f10: 0114f8b3     	and	a7, s1, a7
   13f14: 00e8c733     	xor	a4, a7, a4
   13f18: 0064d893     	srli	a7, s1, 0x6
   13f1c: 01a49293     	slli	t0, s1, 0x1a
   13f20: 0112e8b3     	or	a7, t0, a7
   13f24: 00b4d293     	srli	t0, s1, 0xb
   13f28: 01549e93     	slli	t4, s1, 0x15
   13f2c: 005ee2b3     	or	t0, t4, t0
   13f30: 0194de93     	srli	t4, s1, 0x19
   13f34: 00749913     	slli	s2, s1, 0x7
   13f38: 01d96eb3     	or	t4, s2, t4
   13f3c: 0026d913     	srli	s2, a3, 0x2
   13f40: 01e69a13     	slli	s4, a3, 0x1e
   13f44: 012a6933     	or	s2, s4, s2
   13f48: 00d6da13     	srli	s4, a3, 0xd
   13f4c: 01369a93     	slli	s5, a3, 0x13
   13f50: 014aea33     	or	s4, s5, s4
   13f54: 0166da93     	srli	s5, a3, 0x16
   13f58: 00a69b93     	slli	s7, a3, 0xa
   13f5c: 015beab3     	or	s5, s7, s5
   13f60: 00a64bb3     	xor	s7, a2, a0
   13f64: 0176fbb3     	and	s7, a3, s7
   13f68: 00a67cb3     	and	s9, a2, a0
   13f6c: 019bcbb3     	xor	s7, s7, s9
   13f70: 00e98733     	add	a4, s3, a4
   13f74: 0058c8b3     	xor	a7, a7, t0
   13f78: 014942b3     	xor	t0, s2, s4
   13f7c: 01d8c8b3     	xor	a7, a7, t4
   13f80: d5a79eb7     	lui	t4, 0xd5a79
   13f84: 147e8e93     	addi	t4, t4, 0x147
   13f88: 01d70733     	add	a4, a4, t4
   13f8c: 0152c2b3     	xor	t0, t0, s5
   13f90: 011708b3     	add	a7, a4, a7
   13f94: 017282b3     	add	t0, t0, s7
   13f98: 00b88733     	add	a4, a7, a1
   13f9c: 011285b3     	add	a1, t0, a7
   13fa0: 0e812983     	lw	s3, 0xe8(sp)
   13fa4: 00f989b3     	add	s3, s3, a5
   13fa8: 00f4c8b3     	xor	a7, s1, a5
   13fac: 011778b3     	and	a7, a4, a7
   13fb0: 00f8c7b3     	xor	a5, a7, a5
   13fb4: 00675893     	srli	a7, a4, 0x6
   13fb8: 01a71293     	slli	t0, a4, 0x1a
   13fbc: 0112e8b3     	or	a7, t0, a7
   13fc0: 00b75293     	srli	t0, a4, 0xb
   13fc4: 01571e93     	slli	t4, a4, 0x15
   13fc8: 005ee2b3     	or	t0, t4, t0
   13fcc: 01975e93     	srli	t4, a4, 0x19
   13fd0: 00771913     	slli	s2, a4, 0x7
   13fd4: 01d96eb3     	or	t4, s2, t4
   13fd8: 0025d913     	srli	s2, a1, 0x2
   13fdc: 01e59a13     	slli	s4, a1, 0x1e
   13fe0: 012a6933     	or	s2, s4, s2
   13fe4: 00d5da13     	srli	s4, a1, 0xd
   13fe8: 01359a93     	slli	s5, a1, 0x13
   13fec: 014aea33     	or	s4, s5, s4
   13ff0: 0165da93     	srli	s5, a1, 0x16
   13ff4: 00a59b93     	slli	s7, a1, 0xa
   13ff8: 015beab3     	or	s5, s7, s5
   13ffc: 00c6cbb3     	xor	s7, a3, a2
   14000: 0175fbb3     	and	s7, a1, s7
   14004: 00c6fcb3     	and	s9, a3, a2
   14008: 019bcbb3     	xor	s7, s7, s9
   1400c: 00f807b3     	add	a5, a6, a5
   14010: 0058c833     	xor	a6, a7, t0
   14014: 014948b3     	xor	a7, s2, s4
   14018: 01d84833     	xor	a6, a6, t4
   1401c: 06ca62b7     	lui	t0, 0x6ca6
   14020: 35128293     	addi	t0, t0, 0x351
   14024: 005787b3     	add	a5, a5, t0
   14028: 0158c8b3     	xor	a7, a7, s5
   1402c: 01078833     	add	a6, a5, a6
   14030: 017888b3     	add	a7, a7, s7
   14034: 00a807b3     	add	a5, a6, a0
   14038: 01088533     	add	a0, a7, a6
   1403c: 07c12803     	lw	a6, 0x7c(sp)
   14040: 00980833     	add	a6, a6, s1
   14044: 009748b3     	xor	a7, a4, s1
   14048: 0117f8b3     	and	a7, a5, a7
   1404c: 0098c8b3     	xor	a7, a7, s1
   14050: 0067d293     	srli	t0, a5, 0x6
   14054: 01a79e93     	slli	t4, a5, 0x1a
   14058: 005ee2b3     	or	t0, t4, t0
   1405c: 00b7de93     	srli	t4, a5, 0xb
   14060: 01579493     	slli	s1, a5, 0x15
   14064: 01d4eeb3     	or	t4, s1, t4
   14068: 0197d493     	srli	s1, a5, 0x19
   1406c: 00779913     	slli	s2, a5, 0x7
   14070: 009964b3     	or	s1, s2, s1
   14074: 00255913     	srli	s2, a0, 0x2
   14078: 01e51a13     	slli	s4, a0, 0x1e
   1407c: 012a6933     	or	s2, s4, s2
   14080: 00d55a13     	srli	s4, a0, 0xd
   14084: 01351a93     	slli	s5, a0, 0x13
   14088: 014aea33     	or	s4, s5, s4
   1408c: 01655a93     	srli	s5, a0, 0x16
   14090: 00a51b93     	slli	s7, a0, 0xa
   14094: 015beab3     	or	s5, s7, s5
   14098: 00d5cbb3     	xor	s7, a1, a3
   1409c: 01757bb3     	and	s7, a0, s7
   140a0: 00d5fcb3     	and	s9, a1, a3
   140a4: 019bcbb3     	xor	s7, s7, s9
   140a8: 011988b3     	add	a7, s3, a7
   140ac: 01d2c2b3     	xor	t0, t0, t4
   140b0: 01494eb3     	xor	t4, s2, s4
   140b4: 0092c2b3     	xor	t0, t0, s1
   140b8: 142934b7     	lui	s1, 0x14293
   140bc: 96748493     	addi	s1, s1, -0x699
   140c0: 009888b3     	add	a7, a7, s1
   140c4: 015eceb3     	xor	t4, t4, s5
   140c8: 005888b3     	add	a7, a7, t0
   140cc: 017e8eb3     	add	t4, t4, s7
   140d0: 00c884b3     	add	s1, a7, a2
   140d4: 011e8633     	add	a2, t4, a7
   140d8: 0c812983     	lw	s3, 0xc8(sp)
   140dc: 00e989b3     	add	s3, s3, a4
   140e0: 00e7c8b3     	xor	a7, a5, a4
   140e4: 0114f8b3     	and	a7, s1, a7
   140e8: 00e8c733     	xor	a4, a7, a4
   140ec: 0064d893     	srli	a7, s1, 0x6
   140f0: 01a49293     	slli	t0, s1, 0x1a
   140f4: 0112e8b3     	or	a7, t0, a7
   140f8: 00b4d293     	srli	t0, s1, 0xb
   140fc: 01549e93     	slli	t4, s1, 0x15
   14100: 005ee2b3     	or	t0, t4, t0
   14104: 0194de93     	srli	t4, s1, 0x19
   14108: 00749913     	slli	s2, s1, 0x7
   1410c: 01d96eb3     	or	t4, s2, t4
   14110: 00265913     	srli	s2, a2, 0x2
   14114: 01e61a13     	slli	s4, a2, 0x1e
   14118: 012a6933     	or	s2, s4, s2
   1411c: 00d65a13     	srli	s4, a2, 0xd
   14120: 01361a93     	slli	s5, a2, 0x13
   14124: 014aea33     	or	s4, s5, s4
   14128: 01665a93     	srli	s5, a2, 0x16
   1412c: 00a61b93     	slli	s7, a2, 0xa
   14130: 015beab3     	or	s5, s7, s5
   14134: 00b54bb3     	xor	s7, a0, a1
   14138: 01767bb3     	and	s7, a2, s7
   1413c: 00b57cb3     	and	s9, a0, a1
   14140: 019bcbb3     	xor	s7, s7, s9
   14144: 00e80733     	add	a4, a6, a4
   14148: 0058c833     	xor	a6, a7, t0
   1414c: 014948b3     	xor	a7, s2, s4
   14150: 01d84833     	xor	a6, a6, t4
   14154: 27b712b7     	lui	t0, 0x27b71
   14158: a8528293     	addi	t0, t0, -0x57b
   1415c: 00570733     	add	a4, a4, t0
   14160: 0158c8b3     	xor	a7, a7, s5
   14164: 01070833     	add	a6, a4, a6
   14168: 017888b3     	add	a7, a7, s7
   1416c: 00d80733     	add	a4, a6, a3
   14170: 010886b3     	add	a3, a7, a6
   14174: 08012803     	lw	a6, 0x80(sp)
   14178: 00f80833     	add	a6, a6, a5
   1417c: 00f4c8b3     	xor	a7, s1, a5
   14180: 011778b3     	and	a7, a4, a7
   14184: 00f8c7b3     	xor	a5, a7, a5
   14188: 00675893     	srli	a7, a4, 0x6
   1418c: 01a71293     	slli	t0, a4, 0x1a
   14190: 0112e8b3     	or	a7, t0, a7
   14194: 00b75293     	srli	t0, a4, 0xb
   14198: 01571e93     	slli	t4, a4, 0x15
   1419c: 005ee2b3     	or	t0, t4, t0
   141a0: 01975e93     	srli	t4, a4, 0x19
   141a4: 00771913     	slli	s2, a4, 0x7
   141a8: 01d96eb3     	or	t4, s2, t4
   141ac: 0026d913     	srli	s2, a3, 0x2
   141b0: 01e69a13     	slli	s4, a3, 0x1e
   141b4: 012a6933     	or	s2, s4, s2
   141b8: 00d6da13     	srli	s4, a3, 0xd
   141bc: 01369a93     	slli	s5, a3, 0x13
   141c0: 014aea33     	or	s4, s5, s4
   141c4: 0166da93     	srli	s5, a3, 0x16
   141c8: 00a69b93     	slli	s7, a3, 0xa
   141cc: 015beab3     	or	s5, s7, s5
   141d0: 00a64bb3     	xor	s7, a2, a0
   141d4: 0176fbb3     	and	s7, a3, s7
   141d8: 00a67cb3     	and	s9, a2, a0
   141dc: 019bcbb3     	xor	s7, s7, s9
   141e0: 00f987b3     	add	a5, s3, a5
   141e4: 0058c8b3     	xor	a7, a7, t0
   141e8: 014942b3     	xor	t0, s2, s4
   141ec: 01d8c8b3     	xor	a7, a7, t4
   141f0: 2e1b2eb7     	lui	t4, 0x2e1b2
   141f4: 138e8e93     	addi	t4, t4, 0x138
   141f8: 01d787b3     	add	a5, a5, t4
   141fc: 0152c2b3     	xor	t0, t0, s5
   14200: 011788b3     	add	a7, a5, a7
   14204: 017282b3     	add	t0, t0, s7
   14208: 00b887b3     	add	a5, a7, a1
   1420c: 011285b3     	add	a1, t0, a7
   14210: 0b012983     	lw	s3, 0xb0(sp)
   14214: 009989b3     	add	s3, s3, s1
   14218: 009748b3     	xor	a7, a4, s1
   1421c: 0117f8b3     	and	a7, a5, a7
   14220: 0098c8b3     	xor	a7, a7, s1
   14224: 0067d293     	srli	t0, a5, 0x6
   14228: 01a79e93     	slli	t4, a5, 0x1a
   1422c: 005ee2b3     	or	t0, t4, t0
   14230: 00b7de93     	srli	t4, a5, 0xb
   14234: 01579493     	slli	s1, a5, 0x15
   14238: 01d4eeb3     	or	t4, s1, t4
   1423c: 0197d493     	srli	s1, a5, 0x19
   14240: 00779913     	slli	s2, a5, 0x7
   14244: 009964b3     	or	s1, s2, s1
   14248: 0025d913     	srli	s2, a1, 0x2
   1424c: 01e59a13     	slli	s4, a1, 0x1e
   14250: 012a6933     	or	s2, s4, s2
   14254: 00d5da13     	srli	s4, a1, 0xd
   14258: 01359a93     	slli	s5, a1, 0x13
   1425c: 014aea33     	or	s4, s5, s4
   14260: 0165da93     	srli	s5, a1, 0x16
   14264: 00a59b93     	slli	s7, a1, 0xa
   14268: 015beab3     	or	s5, s7, s5
   1426c: 00c6cbb3     	xor	s7, a3, a2
   14270: 0175fbb3     	and	s7, a1, s7
   14274: 00c6fcb3     	and	s9, a3, a2
   14278: 019bcbb3     	xor	s7, s7, s9
   1427c: 01180833     	add	a6, a6, a7
   14280: 01d2c8b3     	xor	a7, t0, t4
   14284: 014942b3     	xor	t0, s2, s4
   14288: 0098c8b3     	xor	a7, a7, s1
   1428c: 4d2c7eb7     	lui	t4, 0x4d2c7
   14290: dfce8e93     	addi	t4, t4, -0x204
   14294: 01d80833     	add	a6, a6, t4
   14298: 0152c2b3     	xor	t0, t0, s5
   1429c: 01180833     	add	a6, a6, a7
   142a0: 017282b3     	add	t0, t0, s7
   142a4: 00a804b3     	add	s1, a6, a0
   142a8: 01028533     	add	a0, t0, a6
   142ac: 07412803     	lw	a6, 0x74(sp)
   142b0: 00e80833     	add	a6, a6, a4
   142b4: 00e7c8b3     	xor	a7, a5, a4
   142b8: 0114f8b3     	and	a7, s1, a7
   142bc: 00e8c733     	xor	a4, a7, a4
   142c0: 0064d893     	srli	a7, s1, 0x6
   142c4: 01a49293     	slli	t0, s1, 0x1a
   142c8: 0112e8b3     	or	a7, t0, a7
   142cc: 00b4d293     	srli	t0, s1, 0xb
   142d0: 01549e93     	slli	t4, s1, 0x15
   142d4: 005ee2b3     	or	t0, t4, t0
   142d8: 0194de93     	srli	t4, s1, 0x19
   142dc: 00749913     	slli	s2, s1, 0x7
   142e0: 01d96eb3     	or	t4, s2, t4
   142e4: 00255913     	srli	s2, a0, 0x2
   142e8: 01e51a13     	slli	s4, a0, 0x1e
   142ec: 012a6933     	or	s2, s4, s2
   142f0: 00d55a13     	srli	s4, a0, 0xd
   142f4: 01351a93     	slli	s5, a0, 0x13
   142f8: 014aea33     	or	s4, s5, s4
   142fc: 01655a93     	srli	s5, a0, 0x16
   14300: 00a51b93     	slli	s7, a0, 0xa
   14304: 015beab3     	or	s5, s7, s5
   14308: 00d5cbb3     	xor	s7, a1, a3
   1430c: 01757bb3     	and	s7, a0, s7
   14310: 00d5fcb3     	and	s9, a1, a3
   14314: 019bcbb3     	xor	s7, s7, s9
   14318: 00e98733     	add	a4, s3, a4
   1431c: 0058c8b3     	xor	a7, a7, t0
   14320: 014942b3     	xor	t0, s2, s4
   14324: 01d8c8b3     	xor	a7, a7, t4
   14328: 53381eb7     	lui	t4, 0x53381
   1432c: d13e8e93     	addi	t4, t4, -0x2ed
   14330: 01d70733     	add	a4, a4, t4
   14334: 0152c2b3     	xor	t0, t0, s5
   14338: 011708b3     	add	a7, a4, a7
   1433c: 017282b3     	add	t0, t0, s7
   14340: 00c88733     	add	a4, a7, a2
   14344: 01128633     	add	a2, t0, a7
   14348: 0bc12983     	lw	s3, 0xbc(sp)
   1434c: 00f989b3     	add	s3, s3, a5
   14350: 00f4c8b3     	xor	a7, s1, a5
   14354: 011778b3     	and	a7, a4, a7
   14358: 00f8c7b3     	xor	a5, a7, a5
   1435c: 00675893     	srli	a7, a4, 0x6
   14360: 01a71293     	slli	t0, a4, 0x1a
   14364: 0112e8b3     	or	a7, t0, a7
   14368: 00b75293     	srli	t0, a4, 0xb
   1436c: 01571e93     	slli	t4, a4, 0x15
   14370: 005ee2b3     	or	t0, t4, t0
   14374: 01975e93     	srli	t4, a4, 0x19
   14378: 00771913     	slli	s2, a4, 0x7
   1437c: 01d96eb3     	or	t4, s2, t4
   14380: 00265913     	srli	s2, a2, 0x2
   14384: 01e61a13     	slli	s4, a2, 0x1e
   14388: 012a6933     	or	s2, s4, s2
   1438c: 00d65a13     	srli	s4, a2, 0xd
   14390: 01361a93     	slli	s5, a2, 0x13
   14394: 014aea33     	or	s4, s5, s4
   14398: 01665a93     	srli	s5, a2, 0x16
   1439c: 00a61b93     	slli	s7, a2, 0xa
   143a0: 015beab3     	or	s5, s7, s5
   143a4: 00b54bb3     	xor	s7, a0, a1
   143a8: 01767bb3     	and	s7, a2, s7
   143ac: 00b57cb3     	and	s9, a0, a1
   143b0: 019bcbb3     	xor	s7, s7, s9
   143b4: 00f807b3     	add	a5, a6, a5
   143b8: 0058c833     	xor	a6, a7, t0
   143bc: 014948b3     	xor	a7, s2, s4
   143c0: 01d84833     	xor	a6, a6, t4
   143c4: 650a72b7     	lui	t0, 0x650a7
   143c8: 35428293     	addi	t0, t0, 0x354
   143cc: 005787b3     	add	a5, a5, t0
   143d0: 0158c8b3     	xor	a7, a7, s5
   143d4: 01078833     	add	a6, a5, a6
   143d8: 017888b3     	add	a7, a7, s7
   143dc: 00d807b3     	add	a5, a6, a3
   143e0: 010886b3     	add	a3, a7, a6
   143e4: 07012803     	lw	a6, 0x70(sp)
   143e8: 00980833     	add	a6, a6, s1
   143ec: 009748b3     	xor	a7, a4, s1
   143f0: 0117f8b3     	and	a7, a5, a7
   143f4: 0098c8b3     	xor	a7, a7, s1
   143f8: 0067d293     	srli	t0, a5, 0x6
   143fc: 01a79e93     	slli	t4, a5, 0x1a
   14400: 005ee2b3     	or	t0, t4, t0
   14404: 00b7de93     	srli	t4, a5, 0xb
   14408: 01579493     	slli	s1, a5, 0x15
   1440c: 01d4eeb3     	or	t4, s1, t4
   14410: 0197d493     	srli	s1, a5, 0x19
   14414: 00779913     	slli	s2, a5, 0x7
   14418: 009964b3     	or	s1, s2, s1
   1441c: 0026d913     	srli	s2, a3, 0x2
   14420: 01e69a13     	slli	s4, a3, 0x1e
   14424: 012a6933     	or	s2, s4, s2
   14428: 00d6da13     	srli	s4, a3, 0xd
   1442c: 01369a93     	slli	s5, a3, 0x13
   14430: 014aea33     	or	s4, s5, s4
   14434: 0166da93     	srli	s5, a3, 0x16
   14438: 00a69b93     	slli	s7, a3, 0xa
   1443c: 015beab3     	or	s5, s7, s5
   14440: 00a64bb3     	xor	s7, a2, a0
   14444: 0176fbb3     	and	s7, a3, s7
   14448: 00a67cb3     	and	s9, a2, a0
   1444c: 019bcbb3     	xor	s7, s7, s9
   14450: 011988b3     	add	a7, s3, a7
   14454: 01d2c2b3     	xor	t0, t0, t4
   14458: 01494eb3     	xor	t4, s2, s4
   1445c: 0092c2b3     	xor	t0, t0, s1
   14460: 766a14b7     	lui	s1, 0x766a1
   14464: abb48493     	addi	s1, s1, -0x545
   14468: 009888b3     	add	a7, a7, s1
   1446c: 015eceb3     	xor	t4, t4, s5
   14470: 005888b3     	add	a7, a7, t0
   14474: 017e8eb3     	add	t4, t4, s7
   14478: 00b884b3     	add	s1, a7, a1
   1447c: 011e85b3     	add	a1, t4, a7
   14480: 0b812983     	lw	s3, 0xb8(sp)
   14484: 00e989b3     	add	s3, s3, a4
   14488: 00e7c8b3     	xor	a7, a5, a4
   1448c: 0114f8b3     	and	a7, s1, a7
   14490: 00e8c733     	xor	a4, a7, a4
   14494: 0064d893     	srli	a7, s1, 0x6
   14498: 01a49293     	slli	t0, s1, 0x1a
   1449c: 0112e8b3     	or	a7, t0, a7
   144a0: 00b4d293     	srli	t0, s1, 0xb
   144a4: 01549e93     	slli	t4, s1, 0x15
   144a8: 005ee2b3     	or	t0, t4, t0
   144ac: 0194de93     	srli	t4, s1, 0x19
   144b0: 00749913     	slli	s2, s1, 0x7
   144b4: 01d96eb3     	or	t4, s2, t4
   144b8: 0025d913     	srli	s2, a1, 0x2
   144bc: 01e59a13     	slli	s4, a1, 0x1e
   144c0: 012a6933     	or	s2, s4, s2
   144c4: 00d5da13     	srli	s4, a1, 0xd
   144c8: 01359a93     	slli	s5, a1, 0x13
   144cc: 014aea33     	or	s4, s5, s4
   144d0: 0165da93     	srli	s5, a1, 0x16
   144d4: 00a59b93     	slli	s7, a1, 0xa
   144d8: 015beab3     	or	s5, s7, s5
   144dc: 00c6cbb3     	xor	s7, a3, a2
   144e0: 0175fbb3     	and	s7, a1, s7
   144e4: 00c6fcb3     	and	s9, a3, a2
   144e8: 019bcbb3     	xor	s7, s7, s9
   144ec: 00e80733     	add	a4, a6, a4
   144f0: 0058c833     	xor	a6, a7, t0
   144f4: 014948b3     	xor	a7, s2, s4
   144f8: 01d84833     	xor	a6, a6, t4
   144fc: 81c2d2b7     	lui	t0, 0x81c2d
   14500: 92e28293     	addi	t0, t0, -0x6d2
   14504: 00570733     	add	a4, a4, t0
   14508: 0158c8b3     	xor	a7, a7, s5
   1450c: 01070833     	add	a6, a4, a6
   14510: 017888b3     	add	a7, a7, s7
   14514: 00a80733     	add	a4, a6, a0
   14518: 01088533     	add	a0, a7, a6
   1451c: 06c12803     	lw	a6, 0x6c(sp)
   14520: 00f80833     	add	a6, a6, a5
   14524: 00f4c8b3     	xor	a7, s1, a5
   14528: 011778b3     	and	a7, a4, a7
   1452c: 00f8c7b3     	xor	a5, a7, a5
   14530: 00675893     	srli	a7, a4, 0x6
   14534: 01a71293     	slli	t0, a4, 0x1a
   14538: 0112e8b3     	or	a7, t0, a7
   1453c: 00b75293     	srli	t0, a4, 0xb
   14540: 01571e93     	slli	t4, a4, 0x15
   14544: 005ee2b3     	or	t0, t4, t0
   14548: 01975e93     	srli	t4, a4, 0x19
   1454c: 00771913     	slli	s2, a4, 0x7
   14550: 01d96eb3     	or	t4, s2, t4
   14554: 00255913     	srli	s2, a0, 0x2
   14558: 01e51a13     	slli	s4, a0, 0x1e
   1455c: 012a6933     	or	s2, s4, s2
   14560: 00d55a13     	srli	s4, a0, 0xd
   14564: 01351a93     	slli	s5, a0, 0x13
   14568: 014aea33     	or	s4, s5, s4
   1456c: 01655a93     	srli	s5, a0, 0x16
   14570: 00a51b93     	slli	s7, a0, 0xa
   14574: 015beab3     	or	s5, s7, s5
   14578: 00d5cbb3     	xor	s7, a1, a3
   1457c: 01757bb3     	and	s7, a0, s7
   14580: 00d5fcb3     	and	s9, a1, a3
   14584: 019bcbb3     	xor	s7, s7, s9
   14588: 00f987b3     	add	a5, s3, a5
   1458c: 0058c8b3     	xor	a7, a7, t0
   14590: 014942b3     	xor	t0, s2, s4
   14594: 01d8c8b3     	xor	a7, a7, t4
   14598: 92723eb7     	lui	t4, 0x92723
   1459c: c85e8e93     	addi	t4, t4, -0x37b
   145a0: 01d787b3     	add	a5, a5, t4
   145a4: 0152c2b3     	xor	t0, t0, s5
   145a8: 011788b3     	add	a7, a5, a7
   145ac: 017282b3     	add	t0, t0, s7
   145b0: 00c887b3     	add	a5, a7, a2
   145b4: 01128633     	add	a2, t0, a7
   145b8: 06412983     	lw	s3, 0x64(sp)
   145bc: 009989b3     	add	s3, s3, s1
   145c0: 009748b3     	xor	a7, a4, s1
   145c4: 0117f8b3     	and	a7, a5, a7
   145c8: 0098c8b3     	xor	a7, a7, s1
   145cc: 0067d293     	srli	t0, a5, 0x6
   145d0: 01a79e93     	slli	t4, a5, 0x1a
   145d4: 005ee2b3     	or	t0, t4, t0
   145d8: 00b7de93     	srli	t4, a5, 0xb
   145dc: 01579493     	slli	s1, a5, 0x15
   145e0: 01d4eeb3     	or	t4, s1, t4
   145e4: 0197d493     	srli	s1, a5, 0x19
   145e8: 00779913     	slli	s2, a5, 0x7
   145ec: 009964b3     	or	s1, s2, s1
   145f0: 00265913     	srli	s2, a2, 0x2
   145f4: 01e61a13     	slli	s4, a2, 0x1e
   145f8: 012a6933     	or	s2, s4, s2
   145fc: 00d65a13     	srli	s4, a2, 0xd
   14600: 01361a93     	slli	s5, a2, 0x13
   14604: 014aea33     	or	s4, s5, s4
   14608: 01665a93     	srli	s5, a2, 0x16
   1460c: 00a61b93     	slli	s7, a2, 0xa
   14610: 015beab3     	or	s5, s7, s5
   14614: 00b54bb3     	xor	s7, a0, a1
   14618: 01767bb3     	and	s7, a2, s7
   1461c: 00b57cb3     	and	s9, a0, a1
   14620: 019bcbb3     	xor	s7, s7, s9
   14624: 01180833     	add	a6, a6, a7
   14628: 01d2c8b3     	xor	a7, t0, t4
   1462c: 014942b3     	xor	t0, s2, s4
   14630: 0098c8b3     	xor	a7, a7, s1
   14634: a2bffeb7     	lui	t4, 0xa2bff
   14638: 8a1e8e93     	addi	t4, t4, -0x75f
   1463c: 01d80833     	add	a6, a6, t4
   14640: 0152c2b3     	xor	t0, t0, s5
   14644: 01180833     	add	a6, a6, a7
   14648: 017282b3     	add	t0, t0, s7
   1464c: 00d804b3     	add	s1, a6, a3
   14650: 010286b3     	add	a3, t0, a6
   14654: 0ac12403     	lw	s0, 0xac(sp)
   14658: 00e40433     	add	s0, s0, a4
   1465c: 00e7c833     	xor	a6, a5, a4
   14660: 0104f833     	and	a6, s1, a6
   14664: 00e84733     	xor	a4, a6, a4
   14668: 0064d813     	srli	a6, s1, 0x6
   1466c: 01a49893     	slli	a7, s1, 0x1a
   14670: 0108e833     	or	a6, a7, a6
   14674: 00b4d893     	srli	a7, s1, 0xb
   14678: 01549293     	slli	t0, s1, 0x15
   1467c: 0112e8b3     	or	a7, t0, a7
   14680: 0194d293     	srli	t0, s1, 0x19
   14684: 00749e93     	slli	t4, s1, 0x7
   14688: 005ee2b3     	or	t0, t4, t0
   1468c: 0026de93     	srli	t4, a3, 0x2
   14690: 01e69913     	slli	s2, a3, 0x1e
   14694: 01d96eb3     	or	t4, s2, t4
   14698: 00d6d913     	srli	s2, a3, 0xd
   1469c: 01369a13     	slli	s4, a3, 0x13
   146a0: 012a6933     	or	s2, s4, s2
   146a4: 0166da13     	srli	s4, a3, 0x16
   146a8: 00a69a93     	slli	s5, a3, 0xa
   146ac: 014aea33     	or	s4, s5, s4
   146b0: 00a64ab3     	xor	s5, a2, a0
   146b4: 0156fab3     	and	s5, a3, s5
   146b8: 00a67bb3     	and	s7, a2, a0
   146bc: 017acab3     	xor	s5, s5, s7
   146c0: 00e98733     	add	a4, s3, a4
   146c4: 01184833     	xor	a6, a6, a7
   146c8: 012ec8b3     	xor	a7, t4, s2
   146cc: 00584833     	xor	a6, a6, t0
   146d0: a81a62b7     	lui	t0, 0xa81a6
   146d4: 64b28293     	addi	t0, t0, 0x64b
   146d8: 00570733     	add	a4, a4, t0
   146dc: 0148c8b3     	xor	a7, a7, s4
   146e0: 01070833     	add	a6, a4, a6
   146e4: 015888b3     	add	a7, a7, s5
   146e8: 00b80733     	add	a4, a6, a1
   146ec: 010885b3     	add	a1, a7, a6
   146f0: 00fb0b33     	add	s6, s6, a5
   146f4: 00f4c833     	xor	a6, s1, a5
   146f8: 01077833     	and	a6, a4, a6
   146fc: 00f847b3     	xor	a5, a6, a5
   14700: 00675813     	srli	a6, a4, 0x6
   14704: 01a71893     	slli	a7, a4, 0x1a
   14708: 0108e833     	or	a6, a7, a6
   1470c: 00b75893     	srli	a7, a4, 0xb
   14710: 01571293     	slli	t0, a4, 0x15
   14714: 0112e8b3     	or	a7, t0, a7
   14718: 01975293     	srli	t0, a4, 0x19
   1471c: 00771e93     	slli	t4, a4, 0x7
   14720: 005ee2b3     	or	t0, t4, t0
   14724: 0025de93     	srli	t4, a1, 0x2
   14728: 01e59913     	slli	s2, a1, 0x1e
   1472c: 01d96eb3     	or	t4, s2, t4
   14730: 00d5d913     	srli	s2, a1, 0xd
   14734: 01359993     	slli	s3, a1, 0x13
   14738: 0129e933     	or	s2, s3, s2
   1473c: 0165d993     	srli	s3, a1, 0x16
   14740: 00a59a13     	slli	s4, a1, 0xa
   14744: 013a69b3     	or	s3, s4, s3
   14748: 00c6ca33     	xor	s4, a3, a2
   1474c: 0145fa33     	and	s4, a1, s4
   14750: 00c6fab3     	and	s5, a3, a2
   14754: 015a4a33     	xor	s4, s4, s5
   14758: 00f407b3     	add	a5, s0, a5
   1475c: 01184833     	xor	a6, a6, a7
   14760: 012ec8b3     	xor	a7, t4, s2
   14764: 00584833     	xor	a6, a6, t0
   14768: c24b92b7     	lui	t0, 0xc24b9
   1476c: b7028293     	addi	t0, t0, -0x490
   14770: 005787b3     	add	a5, a5, t0
   14774: 0138c8b3     	xor	a7, a7, s3
   14778: 01078833     	add	a6, a5, a6
   1477c: 014888b3     	add	a7, a7, s4
   14780: 00a807b3     	add	a5, a6, a0
   14784: 01088533     	add	a0, a7, a6
   14788: 0c012803     	lw	a6, 0xc0(sp)
   1478c: 00980833     	add	a6, a6, s1
   14790: 009748b3     	xor	a7, a4, s1
   14794: 0117f8b3     	and	a7, a5, a7
   14798: 0098c8b3     	xor	a7, a7, s1
   1479c: 0067d293     	srli	t0, a5, 0x6
   147a0: 01a79e93     	slli	t4, a5, 0x1a
   147a4: 005ee2b3     	or	t0, t4, t0
   147a8: 00b7de93     	srli	t4, a5, 0xb
   147ac: 01579f13     	slli	t5, a5, 0x15
   147b0: 01df6eb3     	or	t4, t5, t4
   147b4: 0197df13     	srli	t5, a5, 0x19
   147b8: 00779413     	slli	s0, a5, 0x7
   147bc: 01e46f33     	or	t5, s0, t5
   147c0: 00255413     	srli	s0, a0, 0x2
   147c4: 01e51493     	slli	s1, a0, 0x1e
   147c8: 0084e433     	or	s0, s1, s0
   147cc: 00d55493     	srli	s1, a0, 0xd
   147d0: 01351913     	slli	s2, a0, 0x13
   147d4: 009964b3     	or	s1, s2, s1
   147d8: 01655913     	srli	s2, a0, 0x16
   147dc: 00a51993     	slli	s3, a0, 0xa
   147e0: 0129e933     	or	s2, s3, s2
   147e4: 00d5c9b3     	xor	s3, a1, a3
   147e8: 013579b3     	and	s3, a0, s3
   147ec: 00d5fa33     	and	s4, a1, a3
   147f0: 0149c9b3     	xor	s3, s3, s4
   147f4: 011b08b3     	add	a7, s6, a7
   147f8: 01d2c2b3     	xor	t0, t0, t4
   147fc: 00944433     	xor	s0, s0, s1
   14800: 01e2c2b3     	xor	t0, t0, t5
   14804: c76c5eb7     	lui	t4, 0xc76c5
   14808: 1a3e8e93     	addi	t4, t4, 0x1a3
   1480c: 01d888b3     	add	a7, a7, t4
   14810: 01244eb3     	xor	t4, s0, s2
   14814: 005882b3     	add	t0, a7, t0
   14818: 013e8eb3     	add	t4, t4, s3
   1481c: 00c288b3     	add	a7, t0, a2
   14820: 005e8633     	add	a2, t4, t0
   14824: 00ec0f33     	add	t5, s8, a4
   14828: 00e7c2b3     	xor	t0, a5, a4
   1482c: 0058f2b3     	and	t0, a7, t0
   14830: 00e2c733     	xor	a4, t0, a4
   14834: 0068d293     	srli	t0, a7, 0x6
   14838: 01a89e93     	slli	t4, a7, 0x1a
   1483c: 005ee2b3     	or	t0, t4, t0
   14840: 00b8de93     	srli	t4, a7, 0xb
   14844: 01589413     	slli	s0, a7, 0x15
   14848: 01d46eb3     	or	t4, s0, t4
   1484c: 0198d413     	srli	s0, a7, 0x19
   14850: 00789493     	slli	s1, a7, 0x7
   14854: 0084e433     	or	s0, s1, s0
   14858: 00265493     	srli	s1, a2, 0x2
   1485c: 01e61913     	slli	s2, a2, 0x1e
   14860: 009964b3     	or	s1, s2, s1
   14864: 00d65913     	srli	s2, a2, 0xd
   14868: 01361993     	slli	s3, a2, 0x13
   1486c: 0129e933     	or	s2, s3, s2
   14870: 01665993     	srli	s3, a2, 0x16
   14874: 00a61a13     	slli	s4, a2, 0xa
   14878: 013a69b3     	or	s3, s4, s3
   1487c: 00b54a33     	xor	s4, a0, a1
   14880: 01467a33     	and	s4, a2, s4
   14884: 00b57ab3     	and	s5, a0, a1
   14888: 015a4a33     	xor	s4, s4, s5
   1488c: 00e80733     	add	a4, a6, a4
   14890: 01d2c833     	xor	a6, t0, t4
   14894: 0124c2b3     	xor	t0, s1, s2
   14898: 00884833     	xor	a6, a6, s0
   1489c: d192feb7     	lui	t4, 0xd192f
   148a0: 819e8e93     	addi	t4, t4, -0x7e7
   148a4: 01d70733     	add	a4, a4, t4
   148a8: 0132c2b3     	xor	t0, t0, s3
   148ac: 01070833     	add	a6, a4, a6
   148b0: 014282b3     	add	t0, t0, s4
   148b4: 00d80733     	add	a4, a6, a3
   148b8: 010286b3     	add	a3, t0, a6
   148bc: 00fe0833     	add	a6, t3, a5
   148c0: 00f8c2b3     	xor	t0, a7, a5
   148c4: 005772b3     	and	t0, a4, t0
   148c8: 00f2c7b3     	xor	a5, t0, a5
   148cc: 00675293     	srli	t0, a4, 0x6
   148d0: 01a71e93     	slli	t4, a4, 0x1a
   148d4: 005ee2b3     	or	t0, t4, t0
   148d8: 00b75e93     	srli	t4, a4, 0xb
   148dc: 01571413     	slli	s0, a4, 0x15
   148e0: 01d46eb3     	or	t4, s0, t4
   148e4: 01975413     	srli	s0, a4, 0x19
   148e8: 00771493     	slli	s1, a4, 0x7
   148ec: 0084e433     	or	s0, s1, s0
   148f0: 0026d493     	srli	s1, a3, 0x2
   148f4: 01e69913     	slli	s2, a3, 0x1e
   148f8: 009964b3     	or	s1, s2, s1
   148fc: 00d6d913     	srli	s2, a3, 0xd
   14900: 01369993     	slli	s3, a3, 0x13
   14904: 0129e933     	or	s2, s3, s2
   14908: 0166d993     	srli	s3, a3, 0x16
   1490c: 00a69a13     	slli	s4, a3, 0xa
   14910: 013a69b3     	or	s3, s4, s3
   14914: 00a64a33     	xor	s4, a2, a0
   14918: 0146fa33     	and	s4, a3, s4
   1491c: 00a67ab3     	and	s5, a2, a0
   14920: 015a4a33     	xor	s4, s4, s5
   14924: 00ff07b3     	add	a5, t5, a5
   14928: 01d2c2b3     	xor	t0, t0, t4
   1492c: 0124ceb3     	xor	t4, s1, s2
   14930: 0082c2b3     	xor	t0, t0, s0
   14934: d6990f37     	lui	t5, 0xd6990
   14938: 624f0f13     	addi	t5, t5, 0x624
   1493c: 01e787b3     	add	a5, a5, t5
   14940: 013eceb3     	xor	t4, t4, s3
   14944: 005787b3     	add	a5, a5, t0
   14948: 014e8eb3     	add	t4, t4, s4
   1494c: 00b782b3     	add	t0, a5, a1
   14950: 00fe85b3     	add	a1, t4, a5
   14954: 01130e33     	add	t3, t1, a7
   14958: 011747b3     	xor	a5, a4, a7
   1495c: 00f2f7b3     	and	a5, t0, a5
   14960: 0117c7b3     	xor	a5, a5, a7
   14964: 0062d893     	srli	a7, t0, 0x6
   14968: 01a29e93     	slli	t4, t0, 0x1a
   1496c: 011ee8b3     	or	a7, t4, a7
   14970: 00b2de93     	srli	t4, t0, 0xb
   14974: 01529f13     	slli	t5, t0, 0x15
   14978: 01df6eb3     	or	t4, t5, t4
   1497c: 0192df13     	srli	t5, t0, 0x19
   14980: 00729413     	slli	s0, t0, 0x7
   14984: 01e46f33     	or	t5, s0, t5
   14988: 0025d413     	srli	s0, a1, 0x2
   1498c: 01e59493     	slli	s1, a1, 0x1e
   14990: 0084e433     	or	s0, s1, s0
   14994: 00d5d493     	srli	s1, a1, 0xd
   14998: 01359913     	slli	s2, a1, 0x13
   1499c: 009964b3     	or	s1, s2, s1
   149a0: 0165d913     	srli	s2, a1, 0x16
   149a4: 00a59993     	slli	s3, a1, 0xa
   149a8: 0129e933     	or	s2, s3, s2
   149ac: 00c6c9b3     	xor	s3, a3, a2
   149b0: 0135f9b3     	and	s3, a1, s3
   149b4: 00c6fa33     	and	s4, a3, a2
   149b8: 0149c9b3     	xor	s3, s3, s4
   149bc: 00f807b3     	add	a5, a6, a5
   149c0: 01d8c833     	xor	a6, a7, t4
   149c4: 00944433     	xor	s0, s0, s1
   149c8: 01e84833     	xor	a6, a6, t5
   149cc: f40e38b7     	lui	a7, 0xf40e3
   149d0: 58588893     	addi	a7, a7, 0x585
   149d4: 011787b3     	add	a5, a5, a7
   149d8: 012448b3     	xor	a7, s0, s2
   149dc: 010787b3     	add	a5, a5, a6
   149e0: 013888b3     	add	a7, a7, s3
   149e4: 00a78533     	add	a0, a5, a0
   149e8: 00f887b3     	add	a5, a7, a5
   149ec: 04412303     	lw	t1, 0x44(sp)
   149f0: 00e30333     	add	t1, t1, a4
   149f4: 00e2c833     	xor	a6, t0, a4
   149f8: 01057833     	and	a6, a0, a6
   149fc: 00e84733     	xor	a4, a6, a4
   14a00: 00655813     	srli	a6, a0, 0x6
   14a04: 01a51893     	slli	a7, a0, 0x1a
   14a08: 0108e833     	or	a6, a7, a6
   14a0c: 00b55893     	srli	a7, a0, 0xb
   14a10: 01551e93     	slli	t4, a0, 0x15
   14a14: 011ee8b3     	or	a7, t4, a7
   14a18: 01955e93     	srli	t4, a0, 0x19
   14a1c: 00751f13     	slli	t5, a0, 0x7
   14a20: 01df6eb3     	or	t4, t5, t4
   14a24: 0027df13     	srli	t5, a5, 0x2
   14a28: 01e79413     	slli	s0, a5, 0x1e
   14a2c: 01e46f33     	or	t5, s0, t5
   14a30: 00d7d413     	srli	s0, a5, 0xd
   14a34: 01379493     	slli	s1, a5, 0x13
   14a38: 0084e433     	or	s0, s1, s0
   14a3c: 0167d493     	srli	s1, a5, 0x16
   14a40: 00a79913     	slli	s2, a5, 0xa
   14a44: 009964b3     	or	s1, s2, s1
   14a48: 00d5c933     	xor	s2, a1, a3
   14a4c: 0127f933     	and	s2, a5, s2
   14a50: 00d5f9b3     	and	s3, a1, a3
   14a54: 01394933     	xor	s2, s2, s3
   14a58: 00ee0733     	add	a4, t3, a4
   14a5c: 01184833     	xor	a6, a6, a7
   14a60: 008f48b3     	xor	a7, t5, s0
   14a64: 01d84833     	xor	a6, a6, t4
   14a68: 106aae37     	lui	t3, 0x106aa
   14a6c: 070e0e13     	addi	t3, t3, 0x70
   14a70: 01c70733     	add	a4, a4, t3
   14a74: 0098c8b3     	xor	a7, a7, s1
   14a78: 01070833     	add	a6, a4, a6
   14a7c: 012888b3     	add	a7, a7, s2
   14a80: 00c80733     	add	a4, a6, a2
   14a84: 01088633     	add	a2, a7, a6
   14a88: 04012883     	lw	a7, 0x40(sp)
   14a8c: 005888b3     	add	a7, a7, t0
   14a90: 00554833     	xor	a6, a0, t0
   14a94: 01077833     	and	a6, a4, a6
   14a98: 00584833     	xor	a6, a6, t0
   14a9c: 00675293     	srli	t0, a4, 0x6
   14aa0: 01a71e13     	slli	t3, a4, 0x1a
   14aa4: 005e62b3     	or	t0, t3, t0
   14aa8: 00b75e13     	srli	t3, a4, 0xb
   14aac: 01571e93     	slli	t4, a4, 0x15
   14ab0: 01ceee33     	or	t3, t4, t3
   14ab4: 01975e93     	srli	t4, a4, 0x19
   14ab8: 00771f13     	slli	t5, a4, 0x7
   14abc: 01df6eb3     	or	t4, t5, t4
   14ac0: 00265f13     	srli	t5, a2, 0x2
   14ac4: 01e61413     	slli	s0, a2, 0x1e
   14ac8: 01e46f33     	or	t5, s0, t5
   14acc: 00d65413     	srli	s0, a2, 0xd
   14ad0: 01361493     	slli	s1, a2, 0x13
   14ad4: 0084e433     	or	s0, s1, s0
   14ad8: 01665493     	srli	s1, a2, 0x16
   14adc: 00a61913     	slli	s2, a2, 0xa
   14ae0: 009964b3     	or	s1, s2, s1
   14ae4: 00b7c933     	xor	s2, a5, a1
   14ae8: 01267933     	and	s2, a2, s2
   14aec: 00b7f9b3     	and	s3, a5, a1
   14af0: 01394933     	xor	s2, s2, s3
   14af4: 01030833     	add	a6, t1, a6
   14af8: 01c2c2b3     	xor	t0, t0, t3
   14afc: 008f4333     	xor	t1, t5, s0
   14b00: 01d2c2b3     	xor	t0, t0, t4
   14b04: 19a4ce37     	lui	t3, 0x19a4c
   14b08: 116e0e13     	addi	t3, t3, 0x116
   14b0c: 01c80833     	add	a6, a6, t3
   14b10: 00934333     	xor	t1, t1, s1
   14b14: 005802b3     	add	t0, a6, t0
   14b18: 01230333     	add	t1, t1, s2
   14b1c: 00d28833     	add	a6, t0, a3
   14b20: 005306b3     	add	a3, t1, t0
   14b24: 05012e83     	lw	t4, 0x50(sp)
   14b28: 00ae8eb3     	add	t4, t4, a0
   14b2c: 00a742b3     	xor	t0, a4, a0
   14b30: 005872b3     	and	t0, a6, t0
   14b34: 00a2c533     	xor	a0, t0, a0
   14b38: 00685293     	srli	t0, a6, 0x6
   14b3c: 01a81313     	slli	t1, a6, 0x1a
   14b40: 005362b3     	or	t0, t1, t0
   14b44: 00b85313     	srli	t1, a6, 0xb
   14b48: 01581e13     	slli	t3, a6, 0x15
   14b4c: 006e6333     	or	t1, t3, t1
   14b50: 01985e13     	srli	t3, a6, 0x19
   14b54: 00781f13     	slli	t5, a6, 0x7
   14b58: 01cf6e33     	or	t3, t5, t3
   14b5c: 0026df13     	srli	t5, a3, 0x2
   14b60: 01e69413     	slli	s0, a3, 0x1e
   14b64: 01e46f33     	or	t5, s0, t5
   14b68: 00d6d413     	srli	s0, a3, 0xd
   14b6c: 01369493     	slli	s1, a3, 0x13
   14b70: 0084e433     	or	s0, s1, s0
   14b74: 0166d493     	srli	s1, a3, 0x16
   14b78: 00a69913     	slli	s2, a3, 0xa
   14b7c: 009964b3     	or	s1, s2, s1
   14b80: 00f64933     	xor	s2, a2, a5
   14b84: 0126f933     	and	s2, a3, s2
   14b88: 00f679b3     	and	s3, a2, a5
   14b8c: 01394933     	xor	s2, s2, s3
   14b90: 00a88533     	add	a0, a7, a0
   14b94: 0062c8b3     	xor	a7, t0, t1
   14b98: 008f42b3     	xor	t0, t5, s0
   14b9c: 01c8c8b3     	xor	a7, a7, t3
   14ba0: 1e377337     	lui	t1, 0x1e377
   14ba4: c0830313     	addi	t1, t1, -0x3f8
   14ba8: 00650533     	add	a0, a0, t1
   14bac: 0092c2b3     	xor	t0, t0, s1
   14bb0: 01150533     	add	a0, a0, a7
   14bb4: 012282b3     	add	t0, t0, s2
   14bb8: 00b508b3     	add	a7, a0, a1
   14bbc: 00a28533     	add	a0, t0, a0
   14bc0: 0d812a83     	lw	s5, 0xd8(sp)
   14bc4: 00ea8ab3     	add	s5, s5, a4
   14bc8: 00e845b3     	xor	a1, a6, a4
   14bcc: 00b8f5b3     	and	a1, a7, a1
   14bd0: 00e5c5b3     	xor	a1, a1, a4
   14bd4: 0068d713     	srli	a4, a7, 0x6
   14bd8: 01a89293     	slli	t0, a7, 0x1a
   14bdc: 00e2e733     	or	a4, t0, a4
   14be0: 00b8d293     	srli	t0, a7, 0xb
   14be4: 01589313     	slli	t1, a7, 0x15
   14be8: 005362b3     	or	t0, t1, t0
   14bec: 0198d313     	srli	t1, a7, 0x19
   14bf0: 00789e13     	slli	t3, a7, 0x7
   14bf4: 006e6333     	or	t1, t3, t1
   14bf8: 00255e13     	srli	t3, a0, 0x2
   14bfc: 01e51f13     	slli	t5, a0, 0x1e
   14c00: 01cf6e33     	or	t3, t5, t3
   14c04: 00d55f13     	srli	t5, a0, 0xd
   14c08: 01351413     	slli	s0, a0, 0x13
   14c0c: 01e46f33     	or	t5, s0, t5
   14c10: 01655413     	srli	s0, a0, 0x16
   14c14: 00a51493     	slli	s1, a0, 0xa
   14c18: 0084e433     	or	s0, s1, s0
   14c1c: 00c6c4b3     	xor	s1, a3, a2
   14c20: 009574b3     	and	s1, a0, s1
   14c24: 00c6f933     	and	s2, a3, a2
   14c28: 0124c4b3     	xor	s1, s1, s2
   14c2c: 00be85b3     	add	a1, t4, a1
   14c30: 00574733     	xor	a4, a4, t0
   14c34: 01ee42b3     	xor	t0, t3, t5
   14c38: 00674733     	xor	a4, a4, t1
   14c3c: 27487337     	lui	t1, 0x27487
   14c40: 74c30313     	addi	t1, t1, 0x74c
   14c44: 006585b3     	add	a1, a1, t1
   14c48: 0082c2b3     	xor	t0, t0, s0
   14c4c: 00e585b3     	add	a1, a1, a4
   14c50: 009284b3     	add	s1, t0, s1
   14c54: 00f582b3     	add	t0, a1, a5
   14c58: 00b485b3     	add	a1, s1, a1
   14c5c: 05412903     	lw	s2, 0x54(sp)
   14c60: 01090933     	add	s2, s2, a6
   14c64: 0108c733     	xor	a4, a7, a6
   14c68: 00e2f733     	and	a4, t0, a4
   14c6c: 01074733     	xor	a4, a4, a6
   14c70: 0062d793     	srli	a5, t0, 0x6
   14c74: 01a29813     	slli	a6, t0, 0x1a
   14c78: 00f867b3     	or	a5, a6, a5
   14c7c: 00b2d813     	srli	a6, t0, 0xb
   14c80: 01529313     	slli	t1, t0, 0x15
   14c84: 01036833     	or	a6, t1, a6
   14c88: 0192d313     	srli	t1, t0, 0x19
   14c8c: 00729e13     	slli	t3, t0, 0x7
   14c90: 006e6333     	or	t1, t3, t1
   14c94: 0025de13     	srli	t3, a1, 0x2
   14c98: 01e59e93     	slli	t4, a1, 0x1e
   14c9c: 01ceee33     	or	t3, t4, t3
   14ca0: 00d5de93     	srli	t4, a1, 0xd
   14ca4: 01359f13     	slli	t5, a1, 0x13
   14ca8: 01df6eb3     	or	t4, t5, t4
   14cac: 0165df13     	srli	t5, a1, 0x16
   14cb0: 00a59413     	slli	s0, a1, 0xa
   14cb4: 01e46f33     	or	t5, s0, t5
   14cb8: 00d54433     	xor	s0, a0, a3
   14cbc: 0085f433     	and	s0, a1, s0
   14cc0: 00d574b3     	and	s1, a0, a3
   14cc4: 00944433     	xor	s0, s0, s1
   14cc8: 00ea8733     	add	a4, s5, a4
   14ccc: 0107c7b3     	xor	a5, a5, a6
   14cd0: 01de4833     	xor	a6, t3, t4
   14cd4: 0067c7b3     	xor	a5, a5, t1
   14cd8: 34b0c337     	lui	t1, 0x34b0c
   14cdc: cb530313     	addi	t1, t1, -0x34b
   14ce0: 00670733     	add	a4, a4, t1
   14ce4: 01e84833     	xor	a6, a6, t5
   14ce8: 00f70733     	add	a4, a4, a5
   14cec: 00880833     	add	a6, a6, s0
   14cf0: 00c707b3     	add	a5, a4, a2
   14cf4: 00e80633     	add	a2, a6, a4
   14cf8: 0cc12c03     	lw	s8, 0xcc(sp)
   14cfc: 011c0c33     	add	s8, s8, a7
   14d00: 0112c733     	xor	a4, t0, a7
   14d04: 00e7f733     	and	a4, a5, a4
   14d08: 01174733     	xor	a4, a4, a7
   14d0c: 0067d813     	srli	a6, a5, 0x6
   14d10: 01a79893     	slli	a7, a5, 0x1a
   14d14: 0108e833     	or	a6, a7, a6
   14d18: 00b7d893     	srli	a7, a5, 0xb
   14d1c: 01579313     	slli	t1, a5, 0x15
   14d20: 011368b3     	or	a7, t1, a7
   14d24: 0197d313     	srli	t1, a5, 0x19
   14d28: 00779e13     	slli	t3, a5, 0x7
   14d2c: 006e6333     	or	t1, t3, t1
   14d30: 00265e13     	srli	t3, a2, 0x2
   14d34: 01e61e93     	slli	t4, a2, 0x1e
   14d38: 01ceee33     	or	t3, t4, t3
   14d3c: 00d65e93     	srli	t4, a2, 0xd
   14d40: 01361f13     	slli	t5, a2, 0x13
   14d44: 01df6eb3     	or	t4, t5, t4
   14d48: 01665f13     	srli	t5, a2, 0x16
   14d4c: 00a61413     	slli	s0, a2, 0xa
   14d50: 01e46f33     	or	t5, s0, t5
   14d54: 00a5c433     	xor	s0, a1, a0
   14d58: 00867433     	and	s0, a2, s0
   14d5c: 00a5f4b3     	and	s1, a1, a0
   14d60: 00944433     	xor	s0, s0, s1
   14d64: 00e90733     	add	a4, s2, a4
   14d68: 01184833     	xor	a6, a6, a7
   14d6c: 01de48b3     	xor	a7, t3, t4
   14d70: 00684833     	xor	a6, a6, t1
   14d74: 391c1337     	lui	t1, 0x391c1
   14d78: cb330313     	addi	t1, t1, -0x34d
   14d7c: 00670733     	add	a4, a4, t1
   14d80: 01e8c8b3     	xor	a7, a7, t5
   14d84: 01070733     	add	a4, a4, a6
   14d88: 008888b3     	add	a7, a7, s0
   14d8c: 05812b83     	lw	s7, 0x58(sp)
   14d90: 005b8bb3     	add	s7, s7, t0
   14d94: 00d706b3     	add	a3, a4, a3
   14d98: 00e88733     	add	a4, a7, a4
   14d9c: 0057c833     	xor	a6, a5, t0
   14da0: 0106f833     	and	a6, a3, a6
   14da4: 00584833     	xor	a6, a6, t0
   14da8: 0066d893     	srli	a7, a3, 0x6
   14dac: 01a69293     	slli	t0, a3, 0x1a
   14db0: 0112e8b3     	or	a7, t0, a7
   14db4: 00b6d293     	srli	t0, a3, 0xb
   14db8: 01569313     	slli	t1, a3, 0x15
   14dbc: 005362b3     	or	t0, t1, t0
   14dc0: 0196d313     	srli	t1, a3, 0x19
   14dc4: 00769e13     	slli	t3, a3, 0x7
   14dc8: 006e6333     	or	t1, t3, t1
   14dcc: 00275e13     	srli	t3, a4, 0x2
   14dd0: 01e71e93     	slli	t4, a4, 0x1e
   14dd4: 01ceee33     	or	t3, t4, t3
   14dd8: 00d75e93     	srli	t4, a4, 0xd
   14ddc: 01371f13     	slli	t5, a4, 0x13
   14de0: 01df6eb3     	or	t4, t5, t4
   14de4: 01675f13     	srli	t5, a4, 0x16
   14de8: 00a71413     	slli	s0, a4, 0xa
   14dec: 01e46f33     	or	t5, s0, t5
   14df0: 00b64433     	xor	s0, a2, a1
   14df4: 00877433     	and	s0, a4, s0
   14df8: 00b674b3     	and	s1, a2, a1
   14dfc: 00944433     	xor	s0, s0, s1
   14e00: 010c0833     	add	a6, s8, a6
   14e04: 0058c8b3     	xor	a7, a7, t0
   14e08: 01de42b3     	xor	t0, t3, t4
   14e0c: 0068c8b3     	xor	a7, a7, t1
   14e10: 4ed8b337     	lui	t1, 0x4ed8b
   14e14: a4a30313     	addi	t1, t1, -0x5b6
   14e18: 00680833     	add	a6, a6, t1
   14e1c: 01e2c2b3     	xor	t0, t0, t5
   14e20: 011808b3     	add	a7, a6, a7
   14e24: 0e412c83     	lw	s9, 0xe4(sp)
   14e28: 00fc8cb3     	add	s9, s9, a5
   14e2c: 008282b3     	add	t0, t0, s0
   14e30: 00f6c333     	xor	t1, a3, a5
   14e34: 00a88833     	add	a6, a7, a0
   14e38: 01128533     	add	a0, t0, a7
   14e3c: 006878b3     	and	a7, a6, t1
   14e40: 00685293     	srli	t0, a6, 0x6
   14e44: 00f8c7b3     	xor	a5, a7, a5
   14e48: 01a81893     	slli	a7, a6, 0x1a
   14e4c: 0058e8b3     	or	a7, a7, t0
   14e50: 00b85293     	srli	t0, a6, 0xb
   14e54: 01581313     	slli	t1, a6, 0x15
   14e58: 005362b3     	or	t0, t1, t0
   14e5c: 01985313     	srli	t1, a6, 0x19
   14e60: 00781e13     	slli	t3, a6, 0x7
   14e64: 006e6333     	or	t1, t3, t1
   14e68: 00255e13     	srli	t3, a0, 0x2
   14e6c: 01e51e93     	slli	t4, a0, 0x1e
   14e70: 01ceee33     	or	t3, t4, t3
   14e74: 00d55e93     	srli	t4, a0, 0xd
   14e78: 01351f13     	slli	t5, a0, 0x13
   14e7c: 01df6eb3     	or	t4, t5, t4
   14e80: 01655f13     	srli	t5, a0, 0x16
   14e84: 00a51413     	slli	s0, a0, 0xa
   14e88: 01e46f33     	or	t5, s0, t5
   14e8c: 00c74433     	xor	s0, a4, a2
   14e90: 00857433     	and	s0, a0, s0
   14e94: 00c774b3     	and	s1, a4, a2
   14e98: 00944433     	xor	s0, s0, s1
   14e9c: 00fb87b3     	add	a5, s7, a5
   14ea0: 0058c8b3     	xor	a7, a7, t0
   14ea4: 01de42b3     	xor	t0, t3, t4
   14ea8: 0068c8b3     	xor	a7, a7, t1
   14eac: 5b9cd337     	lui	t1, 0x5b9cd
   14eb0: a4f30313     	addi	t1, t1, -0x5b1
   14eb4: 006787b3     	add	a5, a5, t1
   14eb8: 01e2c2b3     	xor	t0, t0, t5
   14ebc: 00d08d33     	add	s10, ra, a3
   14ec0: 011787b3     	add	a5, a5, a7
   14ec4: 00d848b3     	xor	a7, a6, a3
   14ec8: 008282b3     	add	t0, t0, s0
   14ecc: 00b785b3     	add	a1, a5, a1
   14ed0: 00f287b3     	add	a5, t0, a5
   14ed4: 0115f8b3     	and	a7, a1, a7
   14ed8: 0065d293     	srli	t0, a1, 0x6
   14edc: 01a59313     	slli	t1, a1, 0x1a
   14ee0: 00d8c6b3     	xor	a3, a7, a3
   14ee4: 00b5d893     	srli	a7, a1, 0xb
   14ee8: 005362b3     	or	t0, t1, t0
   14eec: 01559313     	slli	t1, a1, 0x15
   14ef0: 011368b3     	or	a7, t1, a7
   14ef4: 0195d313     	srli	t1, a1, 0x19
   14ef8: 00759e13     	slli	t3, a1, 0x7
   14efc: 006e6333     	or	t1, t3, t1
   14f00: 0027de13     	srli	t3, a5, 0x2
   14f04: 01e79e93     	slli	t4, a5, 0x1e
   14f08: 01ceee33     	or	t3, t4, t3
   14f0c: 00d7de93     	srli	t4, a5, 0xd
   14f10: 01379f13     	slli	t5, a5, 0x13
   14f14: 01df6eb3     	or	t4, t5, t4
   14f18: 0167df13     	srli	t5, a5, 0x16
   14f1c: 00a79413     	slli	s0, a5, 0xa
   14f20: 01e46f33     	or	t5, s0, t5
   14f24: 00e54433     	xor	s0, a0, a4
   14f28: 0087f433     	and	s0, a5, s0
   14f2c: 00e574b3     	and	s1, a0, a4
   14f30: 00944433     	xor	s0, s0, s1
   14f34: 00dc86b3     	add	a3, s9, a3
   14f38: 0112c8b3     	xor	a7, t0, a7
   14f3c: 01de4e33     	xor	t3, t3, t4
   14f40: 0068c8b3     	xor	a7, a7, t1
   14f44: 682e72b7     	lui	t0, 0x682e7
   14f48: ff328293     	addi	t0, t0, -0xd
   14f4c: 005686b3     	add	a3, a3, t0
   14f50: 05c12283     	lw	t0, 0x5c(sp)
   14f54: 010282b3     	add	t0, t0, a6
   14f58: 01ee4333     	xor	t1, t3, t5
   14f5c: 0105ce33     	xor	t3, a1, a6
   14f60: 011686b3     	add	a3, a3, a7
   14f64: 00830333     	add	t1, t1, s0
   14f68: 00c688b3     	add	a7, a3, a2
   14f6c: 00d306b3     	add	a3, t1, a3
   14f70: 01c8f633     	and	a2, a7, t3
   14f74: 0068d313     	srli	t1, a7, 0x6
   14f78: 01a89e13     	slli	t3, a7, 0x1a
   14f7c: 00b8de93     	srli	t4, a7, 0xb
   14f80: 01064633     	xor	a2, a2, a6
   14f84: 01589813     	slli	a6, a7, 0x15
   14f88: 006e6333     	or	t1, t3, t1
   14f8c: 0198de13     	srli	t3, a7, 0x19
   14f90: 01d86833     	or	a6, a6, t4
   14f94: 00789e93     	slli	t4, a7, 0x7
   14f98: 01ceee33     	or	t3, t4, t3
   14f9c: 0026de93     	srli	t4, a3, 0x2
   14fa0: 01e69f13     	slli	t5, a3, 0x1e
   14fa4: 01df6eb3     	or	t4, t5, t4
   14fa8: 00d6df13     	srli	t5, a3, 0xd
   14fac: 01369413     	slli	s0, a3, 0x13
   14fb0: 01e46f33     	or	t5, s0, t5
   14fb4: 0166d413     	srli	s0, a3, 0x16
   14fb8: 00a69493     	slli	s1, a3, 0xa
   14fbc: 0084e433     	or	s0, s1, s0
   14fc0: 00a7c4b3     	xor	s1, a5, a0
   14fc4: 0096f4b3     	and	s1, a3, s1
   14fc8: 00a7f933     	and	s2, a5, a0
   14fcc: 0124c4b3     	xor	s1, s1, s2
   14fd0: 00cd0633     	add	a2, s10, a2
   14fd4: 01034833     	xor	a6, t1, a6
   14fd8: 01eec333     	xor	t1, t4, t5
   14fdc: 01c84833     	xor	a6, a6, t3
   14fe0: 748f8e37     	lui	t3, 0x748f8
   14fe4: 2eee0e13     	addi	t3, t3, 0x2ee
   14fe8: 00b383b3     	add	t2, t2, a1
   14fec: 01c60633     	add	a2, a2, t3
   14ff0: 00b8ce33     	xor	t3, a7, a1
   14ff4: 00834333     	xor	t1, t1, s0
   14ff8: 01060633     	add	a2, a2, a6
   14ffc: 00930333     	add	t1, t1, s1
   15000: 00e60833     	add	a6, a2, a4
   15004: 00c30633     	add	a2, t1, a2
   15008: 01c87733     	and	a4, a6, t3
   1500c: 00685313     	srli	t1, a6, 0x6
   15010: 01a81e13     	slli	t3, a6, 0x1a
   15014: 00b85e93     	srli	t4, a6, 0xb
   15018: 01581f13     	slli	t5, a6, 0x15
   1501c: 00b745b3     	xor	a1, a4, a1
   15020: 01985713     	srli	a4, a6, 0x19
   15024: 006e6333     	or	t1, t3, t1
   15028: 00781e13     	slli	t3, a6, 0x7
   1502c: 01df6eb3     	or	t4, t5, t4
   15030: 00265f13     	srli	t5, a2, 0x2
   15034: 00ee6733     	or	a4, t3, a4
   15038: 01e61e13     	slli	t3, a2, 0x1e
   1503c: 01ee6e33     	or	t3, t3, t5
   15040: 00d65f13     	srli	t5, a2, 0xd
   15044: 01361413     	slli	s0, a2, 0x13
   15048: 01e46f33     	or	t5, s0, t5
   1504c: 01665413     	srli	s0, a2, 0x16
   15050: 00a61493     	slli	s1, a2, 0xa
   15054: 0084e433     	or	s0, s1, s0
   15058: 00f6c4b3     	xor	s1, a3, a5
   1505c: 009674b3     	and	s1, a2, s1
   15060: 00f6f933     	and	s2, a3, a5
   15064: 0124c4b3     	xor	s1, s1, s2
   15068: 00b285b3     	add	a1, t0, a1
   1506c: 01d342b3     	xor	t0, t1, t4
   15070: 01ee4333     	xor	t1, t3, t5
   15074: 78a56e37     	lui	t3, 0x78a56
   15078: 36fe0e13     	addi	t3, t3, 0x36f
   1507c: 011d8db3     	add	s11, s11, a7
   15080: 00e2c733     	xor	a4, t0, a4
   15084: 011842b3     	xor	t0, a6, a7
   15088: 01c585b3     	add	a1, a1, t3
   1508c: 00834333     	xor	t1, t1, s0
   15090: 00e585b3     	add	a1, a1, a4
   15094: 00930333     	add	t1, t1, s1
   15098: 00a58733     	add	a4, a1, a0
   1509c: 00b305b3     	add	a1, t1, a1
   150a0: 00577533     	and	a0, a4, t0
   150a4: 00675293     	srli	t0, a4, 0x6
   150a8: 01a71313     	slli	t1, a4, 0x1a
   150ac: 00b75e13     	srli	t3, a4, 0xb
   150b0: 01571e93     	slli	t4, a4, 0x15
   150b4: 01975f13     	srli	t5, a4, 0x19
   150b8: 01154533     	xor	a0, a0, a7
   150bc: 00771893     	slli	a7, a4, 0x7
   150c0: 005362b3     	or	t0, t1, t0
   150c4: 0025d313     	srli	t1, a1, 0x2
   150c8: 01ceee33     	or	t3, t4, t3
   150cc: 01e59e93     	slli	t4, a1, 0x1e
   150d0: 01e8e8b3     	or	a7, a7, t5
   150d4: 00d5df13     	srli	t5, a1, 0xd
   150d8: 006ee333     	or	t1, t4, t1
   150dc: 01359e93     	slli	t4, a1, 0x13
   150e0: 01eeeeb3     	or	t4, t4, t5
   150e4: 0165df13     	srli	t5, a1, 0x16
   150e8: 00a59413     	slli	s0, a1, 0xa
   150ec: 01e46f33     	or	t5, s0, t5
   150f0: 00d64433     	xor	s0, a2, a3
   150f4: 0085f433     	and	s0, a1, s0
   150f8: 00d674b3     	and	s1, a2, a3
   150fc: 00944433     	xor	s0, s0, s1
   15100: 00a38533     	add	a0, t2, a0
   15104: 01c2c2b3     	xor	t0, t0, t3
   15108: 84c883b7     	lui	t2, 0x84c88
   1510c: 81438393     	addi	t2, t2, -0x7ec
   15110: 010f8fb3     	add	t6, t6, a6
   15114: 01d34333     	xor	t1, t1, t4
   15118: 01074e33     	xor	t3, a4, a6
   1511c: 0112c8b3     	xor	a7, t0, a7
   15120: 00750533     	add	a0, a0, t2
   15124: 01e342b3     	xor	t0, t1, t5
   15128: 01150533     	add	a0, a0, a7
   1512c: 008282b3     	add	t0, t0, s0
   15130: 00f507b3     	add	a5, a0, a5
   15134: 00a28533     	add	a0, t0, a0
   15138: 01c7f8b3     	and	a7, a5, t3
   1513c: 0067d293     	srli	t0, a5, 0x6
   15140: 01a79313     	slli	t1, a5, 0x1a
   15144: 00b7d393     	srli	t2, a5, 0xb
   15148: 01579e13     	slli	t3, a5, 0x15
   1514c: 0197de93     	srli	t4, a5, 0x19
   15150: 00779f13     	slli	t5, a5, 0x7
   15154: 0108c833     	xor	a6, a7, a6
   15158: 00255893     	srli	a7, a0, 0x2
   1515c: 005362b3     	or	t0, t1, t0
   15160: 01e51313     	slli	t1, a0, 0x1e
   15164: 007e63b3     	or	t2, t3, t2
   15168: 00d55e13     	srli	t3, a0, 0xd
   1516c: 01df6eb3     	or	t4, t5, t4
   15170: 01351f13     	slli	t5, a0, 0x13
   15174: 011368b3     	or	a7, t1, a7
   15178: 01655313     	srli	t1, a0, 0x16
   1517c: 01cf6e33     	or	t3, t5, t3
   15180: 00a51f13     	slli	t5, a0, 0xa
   15184: 006f6333     	or	t1, t5, t1
   15188: 00c5cf33     	xor	t5, a1, a2
   1518c: 01e57f33     	and	t5, a0, t5
   15190: 00c5f433     	and	s0, a1, a2
   15194: 008f4f33     	xor	t5, t5, s0
   15198: 010d8833     	add	a6, s11, a6
   1519c: 8cc70437     	lui	s0, 0x8cc70
   151a0: 20840413     	addi	s0, s0, 0x208
   151a4: 0c412d83     	lw	s11, 0xc4(sp)
   151a8: 00ed8db3     	add	s11, s11, a4
   151ac: 0072c2b3     	xor	t0, t0, t2
   151b0: 00e7c3b3     	xor	t2, a5, a4
   151b4: 01c8c8b3     	xor	a7, a7, t3
   151b8: 00b54e33     	xor	t3, a0, a1
   151bc: 01d2c2b3     	xor	t0, t0, t4
   151c0: 00880833     	add	a6, a6, s0
   151c4: 0068c8b3     	xor	a7, a7, t1
   151c8: 00580833     	add	a6, a6, t0
   151cc: 01e888b3     	add	a7, a7, t5
   151d0: 00d806b3     	add	a3, a6, a3
   151d4: 010882b3     	add	t0, a7, a6
   151d8: 0076f833     	and	a6, a3, t2
   151dc: 0066d893     	srli	a7, a3, 0x6
   151e0: 01a69313     	slli	t1, a3, 0x1a
   151e4: 00b6d393     	srli	t2, a3, 0xb
   151e8: 01569e93     	slli	t4, a3, 0x15
   151ec: 0196df13     	srli	t5, a3, 0x19
   151f0: 00769413     	slli	s0, a3, 0x7
   151f4: 00e84733     	xor	a4, a6, a4
   151f8: 0022d813     	srli	a6, t0, 0x2
   151fc: 011368b3     	or	a7, t1, a7
   15200: 01e29313     	slli	t1, t0, 0x1e
   15204: 007ee3b3     	or	t2, t4, t2
   15208: 00d2de93     	srli	t4, t0, 0xd
   1520c: 01e46f33     	or	t5, s0, t5
   15210: 01329413     	slli	s0, t0, 0x13
   15214: 01036333     	or	t1, t1, a6
   15218: 0162d813     	srli	a6, t0, 0x16
   1521c: 01d46eb3     	or	t4, s0, t4
   15220: 00a29413     	slli	s0, t0, 0xa
   15224: 01046433     	or	s0, s0, a6
   15228: 00b57833     	and	a6, a0, a1
   1522c: 01c2fe33     	and	t3, t0, t3
   15230: 010e4e33     	xor	t3, t3, a6
   15234: 00ef8733     	add	a4, t6, a4
   15238: 90bf0837     	lui	a6, 0x90bf0
   1523c: ffa80f93     	addi	t6, a6, -0x6
   15240: 0dc12803     	lw	a6, 0xdc(sp)
   15244: 00f80833     	add	a6, a6, a5
   15248: 0078c8b3     	xor	a7, a7, t2
   1524c: 00f6c3b3     	xor	t2, a3, a5
   15250: 01d34333     	xor	t1, t1, t4
   15254: 00a2ceb3     	xor	t4, t0, a0
   15258: 01e8c8b3     	xor	a7, a7, t5
   1525c: 01f70733     	add	a4, a4, t6
   15260: 00834333     	xor	t1, t1, s0
   15264: 01170733     	add	a4, a4, a7
   15268: 01c30333     	add	t1, t1, t3
   1526c: 00c708b3     	add	a7, a4, a2
   15270: 00e30333     	add	t1, t1, a4
   15274: 0078f633     	and	a2, a7, t2
   15278: 0068d713     	srli	a4, a7, 0x6
   1527c: 01a89393     	slli	t2, a7, 0x1a
   15280: 00b8de13     	srli	t3, a7, 0xb
   15284: 01589f13     	slli	t5, a7, 0x15
   15288: 0198df93     	srli	t6, a7, 0x19
   1528c: 00789413     	slli	s0, a7, 0x7
   15290: 00f64633     	xor	a2, a2, a5
   15294: 00235793     	srli	a5, t1, 0x2
   15298: 00e3e3b3     	or	t2, t2, a4
   1529c: 01e31713     	slli	a4, t1, 0x1e
   152a0: 01cf6e33     	or	t3, t5, t3
   152a4: 00d35f13     	srli	t5, t1, 0xd
   152a8: 01f46fb3     	or	t6, s0, t6
   152ac: 01331413     	slli	s0, t1, 0x13
   152b0: 00f764b3     	or	s1, a4, a5
   152b4: 01635713     	srli	a4, t1, 0x16
   152b8: 01e46f33     	or	t5, s0, t5
   152bc: 00a31793     	slli	a5, t1, 0xa
   152c0: 00e7e433     	or	s0, a5, a4
   152c4: 00a2f733     	and	a4, t0, a0
   152c8: 01d377b3     	and	a5, t1, t4
   152cc: 00e7ceb3     	xor	t4, a5, a4
   152d0: 00cd8db3     	add	s11, s11, a2
   152d4: a4507637     	lui	a2, 0xa4507
   152d8: ceb60913     	addi	s2, a2, -0x315
   152dc: 0f012703     	lw	a4, 0xf0(sp)
   152e0: 00e28733     	add	a4, t0, a4
   152e4: 0d412783     	lw	a5, 0xd4(sp)
   152e8: 00d787b3     	add	a5, a5, a3
   152ec: 01c3c3b3     	xor	t2, t2, t3
   152f0: 00d8ce33     	xor	t3, a7, a3
   152f4: 0a412603     	lw	a2, 0xa4(sp)
   152f8: 00c30633     	add	a2, t1, a2
   152fc: 01e4cf33     	xor	t5, s1, t5
   15300: 005344b3     	xor	s1, t1, t0
   15304: 005372b3     	and	t0, t1, t0
   15308: 01f3c3b3     	xor	t2, t2, t6
   1530c: 012d8933     	add	s2, s11, s2
   15310: 008f4f33     	xor	t5, t5, s0
   15314: 007903b3     	add	t2, s2, t2
   15318: 01df0eb3     	add	t4, t5, t4
   1531c: 00b38f33     	add	t5, t2, a1
   15320: 007e83b3     	add	t2, t4, t2
   15324: 01cf7e33     	and	t3, t5, t3
   15328: 006f5e93     	srli	t4, t5, 0x6
   1532c: 01af1f93     	slli	t6, t5, 0x1a
   15330: 00bf5413     	srli	s0, t5, 0xb
   15334: 015f1913     	slli	s2, t5, 0x15
   15338: 019f5993     	srli	s3, t5, 0x19
   1533c: 007f1a13     	slli	s4, t5, 0x7
   15340: 0a012583     	lw	a1, 0xa0(sp)
   15344: 00b385b3     	add	a1, t2, a1
   15348: 00de46b3     	xor	a3, t3, a3
   1534c: 0023de13     	srli	t3, t2, 0x2
   15350: 01dfeeb3     	or	t4, t6, t4
   15354: 01e39f93     	slli	t6, t2, 0x1e
   15358: 00896433     	or	s0, s2, s0
   1535c: 00d3d913     	srli	s2, t2, 0xd
   15360: 013a69b3     	or	s3, s4, s3
   15364: 01339a13     	slli	s4, t2, 0x13
   15368: 01cfee33     	or	t3, t6, t3
   1536c: 0163df93     	srli	t6, t2, 0x16
   15370: 0093f4b3     	and	s1, t2, s1
   15374: 012a6933     	or	s2, s4, s2
   15378: 0063ca33     	xor	s4, t2, t1
   1537c: 0063f333     	and	t1, t2, t1
   15380: 00a39393     	slli	t2, t2, 0xa
   15384: 01f3e3b3     	or	t2, t2, t6
   15388: 0054c2b3     	xor	t0, s1, t0
   1538c: 00d806b3     	add	a3, a6, a3
   15390: 008ec833     	xor	a6, t4, s0
   15394: 012e4e33     	xor	t3, t3, s2
   15398: 01384833     	xor	a6, a6, s3
   1539c: bef9aeb7     	lui	t4, 0xbef9a
   153a0: 3f7e8e93     	addi	t4, t4, 0x3f7
   153a4: 01d686b3     	add	a3, a3, t4
   153a8: 007e43b3     	xor	t2, t3, t2
   153ac: 0a812e03     	lw	t3, 0xa8(sp)
   153b0: 01c88e33     	add	t3, a7, t3
   153b4: 09c12e83     	lw	t4, 0x9c(sp)
   153b8: 01df0eb3     	add	t4, t5, t4
   153bc: 011f4f33     	xor	t5, t5, a7
   153c0: 010686b3     	add	a3, a3, a6
   153c4: 005382b3     	add	t0, t2, t0
   153c8: 00a68533     	add	a0, a3, a0
   153cc: 00d286b3     	add	a3, t0, a3
   153d0: 0f812803     	lw	a6, 0xf8(sp)
   153d4: 01050833     	add	a6, a0, a6
   153d8: 01e572b3     	and	t0, a0, t5
   153dc: 00655393     	srli	t2, a0, 0x6
   153e0: 01a51f13     	slli	t5, a0, 0x1a
   153e4: 00b55f93     	srli	t6, a0, 0xb
   153e8: 0112c8b3     	xor	a7, t0, a7
   153ec: 01551293     	slli	t0, a0, 0x15
   153f0: 007f63b3     	or	t2, t5, t2
   153f4: 01955f13     	srli	t5, a0, 0x19
   153f8: 00751513     	slli	a0, a0, 0x7
   153fc: 09812403     	lw	s0, 0x98(sp)
   15400: 00868433     	add	s0, a3, s0
   15404: 0146f4b3     	and	s1, a3, s4
   15408: 01f2e2b3     	or	t0, t0, t6
   1540c: 0026df93     	srli	t6, a3, 0x2
   15410: 01e56533     	or	a0, a0, t5
   15414: 01e69f13     	slli	t5, a3, 0x1e
   15418: 0064c333     	xor	t1, s1, t1
   1541c: 00d6d493     	srli	s1, a3, 0xd
   15420: 01ff6f33     	or	t5, t5, t6
   15424: 01369f93     	slli	t6, a3, 0x13
   15428: 009fefb3     	or	t6, t6, s1
   1542c: 0166d493     	srli	s1, a3, 0x16
   15430: 00a69693     	slli	a3, a3, 0xa
   15434: 0096e6b3     	or	a3, a3, s1
   15438: 011787b3     	add	a5, a5, a7
   1543c: 0053c8b3     	xor	a7, t2, t0
   15440: 0f412283     	lw	t0, 0xf4(sp)
   15444: 00530333     	add	t1, t1, t0
   15448: 01ff42b3     	xor	t0, t5, t6
   1544c: 00a8c533     	xor	a0, a7, a0
   15450: c67188b7     	lui	a7, 0xc6718
   15454: 8f288893     	addi	a7, a7, -0x70e
   15458: 011787b3     	add	a5, a5, a7
   1545c: 00d2c6b3     	xor	a3, t0, a3
   15460: 00a78533     	add	a0, a5, a0
   15464: 00d306b3     	add	a3, t1, a3
   15468: 00a70733     	add	a4, a4, a0
   1546c: 00a68533     	add	a0, a3, a0
   15470: 06012683     	lw	a3, 0x60(sp)
   15474: 00a6a023     	sw	a0, 0x0(a3)
   15478: 0086a223     	sw	s0, 0x4(a3)
   1547c: 00b6a423     	sw	a1, 0x8(a3)
   15480: 00c6a623     	sw	a2, 0xc(a3)
   15484: 00e6a823     	sw	a4, 0x10(a3)
   15488: 0106aa23     	sw	a6, 0x14(a3)
   1548c: 01d6ac23     	sw	t4, 0x18(a3)
   15490: 01c6ae23     	sw	t3, 0x1c(a3)
   15494: 12c12083     	lw	ra, 0x12c(sp)
   15498: 12812403     	lw	s0, 0x128(sp)
   1549c: 12412483     	lw	s1, 0x124(sp)
   154a0: 12012903     	lw	s2, 0x120(sp)
   154a4: 11c12983     	lw	s3, 0x11c(sp)
   154a8: 11812a03     	lw	s4, 0x118(sp)
   154ac: 11412a83     	lw	s5, 0x114(sp)
   154b0: 11012b03     	lw	s6, 0x110(sp)
   154b4: 10c12b83     	lw	s7, 0x10c(sp)
   154b8: 10812c03     	lw	s8, 0x108(sp)
   154bc: 10412c83     	lw	s9, 0x104(sp)
   154c0: 10012d03     	lw	s10, 0x100(sp)
   154c4: 0fc12d83     	lw	s11, 0xfc(sp)
   154c8: 13010113     	addi	sp, sp, 0x130
   154cc: 00008067     	ret

000154d0 <memcpy>:
   154d0: ff010113     	addi	sp, sp, -0x10
   154d4: 00112623     	sw	ra, 0xc(sp)
   154d8: 00812423     	sw	s0, 0x8(sp)
   154dc: 01010413     	addi	s0, sp, 0x10
   154e0: 00c12083     	lw	ra, 0xc(sp)
   154e4: 00812403     	lw	s0, 0x8(sp)
   154e8: 01010113     	addi	sp, sp, 0x10
   154ec: 00000317     	auipc	t1, 0x0
   154f0: 00830067     	jr	0x8(t1) <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c>

000154f4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c>:
   154f4: fe010113     	addi	sp, sp, -0x20
   154f8: 00112e23     	sw	ra, 0x1c(sp)
   154fc: 00812c23     	sw	s0, 0x18(sp)
   15500: 02010413     	addi	s0, sp, 0x20
   15504: 01000693     	li	a3, 0x10
   15508: 08d66063     	bltu	a2, a3, 0x15588 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x94>
   1550c: 40a006b3     	neg	a3, a0
   15510: 0036f693     	andi	a3, a3, 0x3
   15514: 00d507b3     	add	a5, a0, a3
   15518: 02f57463     	bgeu	a0, a5, 0x15540 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x4c>
   1551c: 00068713     	mv	a4, a3
   15520: 00050813     	mv	a6, a0
   15524: 00058893     	mv	a7, a1
   15528: 0008c283     	lbu	t0, 0x0(a7)
   1552c: fff70713     	addi	a4, a4, -0x1
   15530: 00580023     	sb	t0, 0x0(a6)
   15534: 00180813     	addi	a6, a6, 0x1
   15538: 00188893     	addi	a7, a7, 0x1
   1553c: fe0716e3     	bnez	a4, 0x15528 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x34>
   15540: 00d585b3     	add	a1, a1, a3
   15544: 40d60633     	sub	a2, a2, a3
   15548: ffc67713     	andi	a4, a2, -0x4
   1554c: 0035f893     	andi	a7, a1, 0x3
   15550: 00e786b3     	add	a3, a5, a4
   15554: 06089463     	bnez	a7, 0x155bc <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xc8>
   15558: 00d7fe63     	bgeu	a5, a3, 0x15574 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x80>
   1555c: 00058813     	mv	a6, a1
   15560: 00082883     	lw	a7, 0x0(a6)
   15564: 0117a023     	sw	a7, 0x0(a5)
   15568: 00478793     	addi	a5, a5, 0x4
   1556c: 00480813     	addi	a6, a6, 0x4
   15570: fed7e8e3     	bltu	a5, a3, 0x15560 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x6c>
   15574: 00e585b3     	add	a1, a1, a4
   15578: 00367613     	andi	a2, a2, 0x3
   1557c: 00c68733     	add	a4, a3, a2
   15580: 00e6ea63     	bltu	a3, a4, 0x15594 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   15584: 0280006f     	j	0x155ac <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>
   15588: 00050693     	mv	a3, a0
   1558c: 00c50733     	add	a4, a0, a2
   15590: 00e57e63     	bgeu	a0, a4, 0x155ac <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>
   15594: 0005c703     	lbu	a4, 0x0(a1)
   15598: fff60613     	addi	a2, a2, -0x1
   1559c: 00e68023     	sb	a4, 0x0(a3)
   155a0: 00168693     	addi	a3, a3, 0x1
   155a4: 00158593     	addi	a1, a1, 0x1
   155a8: fe0616e3     	bnez	a2, 0x15594 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   155ac: 01c12083     	lw	ra, 0x1c(sp)
   155b0: 01812403     	lw	s0, 0x18(sp)
   155b4: 02010113     	addi	sp, sp, 0x20
   155b8: 00008067     	ret
   155bc: 00000813     	li	a6, 0x0
   155c0: 00400293     	li	t0, 0x4
   155c4: fe042a23     	sw	zero, -0xc(s0)
   155c8: 41128333     	sub	t1, t0, a7
   155cc: ff440293     	addi	t0, s0, -0xc
   155d0: 00137393     	andi	t2, t1, 0x1
   155d4: 0112e2b3     	or	t0, t0, a7
   155d8: 04039e63     	bnez	t2, 0x15634 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x140>
   155dc: 00237313     	andi	t1, t1, 0x2
   155e0: 06031463     	bnez	t1, 0x15648 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x154>
   155e4: ff442e83     	lw	t4, -0xc(s0)
   155e8: 00389813     	slli	a6, a7, 0x3
   155ec: 00478293     	addi	t0, a5, 0x4
   155f0: 41158f33     	sub	t5, a1, a7
   155f4: 06d2fc63     	bgeu	t0, a3, 0x1566c <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x178>
   155f8: 410002b3     	neg	t0, a6
   155fc: 0182fe13     	andi	t3, t0, 0x18
   15600: 004f2283     	lw	t0, 0x4(t5)
   15604: 004f0393     	addi	t2, t5, 0x4
   15608: 010edeb3     	srl	t4, t4, a6
   1560c: 00478313     	addi	t1, a5, 0x4
   15610: 01c29f33     	sll	t5, t0, t3
   15614: 01df6eb3     	or	t4, t5, t4
   15618: 00878f93     	addi	t6, a5, 0x8
   1561c: 01d7a023     	sw	t4, 0x0(a5)
   15620: 00030793     	mv	a5, t1
   15624: 00038f13     	mv	t5, t2
   15628: 00028e93     	mv	t4, t0
   1562c: fcdfeae3     	bltu	t6, a3, 0x15600 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x10c>
   15630: 0480006f     	j	0x15678 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x184>
   15634: 0005c803     	lbu	a6, 0x0(a1)
   15638: 01028023     	sb	a6, 0x0(t0)
   1563c: 00100813     	li	a6, 0x1
   15640: 00237313     	andi	t1, t1, 0x2
   15644: fa0300e3     	beqz	t1, 0x155e4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xf0>
   15648: 01058333     	add	t1, a1, a6
   1564c: 00031303     	lh	t1, 0x0(t1)
   15650: 01028833     	add	a6, t0, a6
   15654: 00681023     	sh	t1, 0x0(a6)
   15658: ff442e83     	lw	t4, -0xc(s0)
   1565c: 00389813     	slli	a6, a7, 0x3
   15660: 00478293     	addi	t0, a5, 0x4
   15664: 41158f33     	sub	t5, a1, a7
   15668: f8d2e8e3     	bltu	t0, a3, 0x155f8 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x104>
   1566c: 000e8293     	mv	t0, t4
   15670: 000f0393     	mv	t2, t5
   15674: 00078313     	mv	t1, a5
   15678: fe040823     	sb	zero, -0x10(s0)
   1567c: 00100793     	li	a5, 0x1
   15680: fe040723     	sb	zero, -0x12(s0)
   15684: 00f89c63     	bne	a7, a5, 0x1569c <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1a8>
   15688: 00000893     	li	a7, 0x0
   1568c: 00000793     	li	a5, 0x0
   15690: 00000e13     	li	t3, 0x0
   15694: ff040e93     	addi	t4, s0, -0x10
   15698: 01c0006f     	j	0x156b4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1c0>
   1569c: 0043c883     	lbu	a7, 0x4(t2)
   156a0: 0053c783     	lbu	a5, 0x5(t2)
   156a4: 00200e13     	li	t3, 0x2
   156a8: ff140823     	sb	a7, -0x10(s0)
   156ac: 00879793     	slli	a5, a5, 0x8
   156b0: fee40e93     	addi	t4, s0, -0x12
   156b4: 0015ff13     	andi	t5, a1, 0x1
   156b8: 000f1663     	bnez	t5, 0x156c4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1d0>
   156bc: 00000393     	li	t2, 0x0
   156c0: 0200006f     	j	0x156e0 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1ec>
   156c4: 00438393     	addi	t2, t2, 0x4
   156c8: 01c383b3     	add	t2, t2, t3
   156cc: 0003c883     	lbu	a7, 0x0(t2)
   156d0: 011e8023     	sb	a7, 0x0(t4)
   156d4: fee44383     	lbu	t2, -0x12(s0)
   156d8: ff044883     	lbu	a7, -0x10(s0)
   156dc: 01039393     	slli	t2, t2, 0x10
   156e0: 0113e8b3     	or	a7, t2, a7
   156e4: 0102d2b3     	srl	t0, t0, a6
   156e8: 41000833     	neg	a6, a6
   156ec: 0117e7b3     	or	a5, a5, a7
   156f0: 01887813     	andi	a6, a6, 0x18
   156f4: 010797b3     	sll	a5, a5, a6
   156f8: 0057e7b3     	or	a5, a5, t0
   156fc: 00f32023     	sw	a5, 0x0(t1)
   15700: 00e585b3     	add	a1, a1, a4
   15704: 00367613     	andi	a2, a2, 0x3
   15708: 00c68733     	add	a4, a3, a2
   1570c: e8e6e4e3     	bltu	a3, a4, 0x15594 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   15710: e9dff06f     	j	0x155ac <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>

00015714 <memset>:
   15714: ff010113     	addi	sp, sp, -0x10
   15718: 00112623     	sw	ra, 0xc(sp)
   1571c: 00812423     	sw	s0, 0x8(sp)
   15720: 01010413     	addi	s0, sp, 0x10
   15724: 01000693     	li	a3, 0x10
   15728: 08d66263     	bltu	a2, a3, 0x157ac <memset+0x98>
   1572c: 40a006b3     	neg	a3, a0
   15730: 0036f693     	andi	a3, a3, 0x3
   15734: 00d50733     	add	a4, a0, a3
   15738: 00e57e63     	bgeu	a0, a4, 0x15754 <memset+0x40>
   1573c: 00068793     	mv	a5, a3
   15740: 00050813     	mv	a6, a0
   15744: 00b80023     	sb	a1, 0x0(a6)
   15748: fff78793     	addi	a5, a5, -0x1
   1574c: 00180813     	addi	a6, a6, 0x1
   15750: fe079ae3     	bnez	a5, 0x15744 <memset+0x30>
   15754: 40d60633     	sub	a2, a2, a3
   15758: ffc67693     	andi	a3, a2, -0x4
   1575c: 00d706b3     	add	a3, a4, a3
   15760: 02d77063     	bgeu	a4, a3, 0x15780 <memset+0x6c>
   15764: 0ff5f793     	andi	a5, a1, 0xff
   15768: 01010837     	lui	a6, 0x1010
   1576c: 10180813     	addi	a6, a6, 0x101
   15770: 030787b3     	mul	a5, a5, a6
   15774: 00f72023     	sw	a5, 0x0(a4)
   15778: 00470713     	addi	a4, a4, 0x4
   1577c: fed76ce3     	bltu	a4, a3, 0x15774 <memset+0x60>
   15780: 00367613     	andi	a2, a2, 0x3
   15784: 00c68733     	add	a4, a3, a2
   15788: 00e6fa63     	bgeu	a3, a4, 0x1579c <memset+0x88>
   1578c: 00b68023     	sb	a1, 0x0(a3)
   15790: fff60613     	addi	a2, a2, -0x1
   15794: 00168693     	addi	a3, a3, 0x1
   15798: fe061ae3     	bnez	a2, 0x1578c <memset+0x78>
   1579c: 00c12083     	lw	ra, 0xc(sp)
   157a0: 00812403     	lw	s0, 0x8(sp)
   157a4: 01010113     	addi	sp, sp, 0x10
   157a8: 00008067     	ret
   157ac: 00050693     	mv	a3, a0
   157b0: 00c50733     	add	a4, a0, a2
   157b4: fce56ce3     	bltu	a0, a4, 0x1578c <memset+0x78>
   157b8: fe5ff06f     	j	0x1579c <memset+0x88>
