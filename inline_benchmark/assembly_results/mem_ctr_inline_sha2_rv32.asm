
inline_benchmark:	file format elf32-littleriscv

Disassembly of section .text:

00011178 <_start>:
   11178: f1010113     	addi	sp, sp, -0xf0
   1117c: 0e112623     	sw	ra, 0xec(sp)
   11180: 0e812423     	sw	s0, 0xe8(sp)
   11184: 0e912223     	sw	s1, 0xe4(sp)
   11188: 0f212023     	sw	s2, 0xe0(sp)
   1118c: 0d312e23     	sw	s3, 0xdc(sp)
   11190: 0d412c23     	sw	s4, 0xd8(sp)
   11194: 0d512a23     	sw	s5, 0xd4(sp)
   11198: 02010513     	addi	a0, sp, 0x20
   1119c: 000105b7     	lui	a1, 0x10
   111a0: 0d458593     	addi	a1, a1, 0xd4
   111a4: 00b00613     	li	a2, 0xb
   111a8: 00b00913     	li	s2, 0xb
   111ac: 00000097     	auipc	ra, 0x0
   111b0: 254080e7     	jalr	0x254(ra) <memcpy>
   111b4: 06010513     	addi	a0, sp, 0x60
   111b8: 00010593     	mv	a1, sp
   111bc: 03000613     	li	a2, 0x30
   111c0: 06010493     	addi	s1, sp, 0x60
   111c4: 00000097     	auipc	ra, 0x0
   111c8: 23c080e7     	jalr	0x23c(ra) <memcpy>
   111cc: 00100513     	li	a0, 0x1
   111d0: 08000593     	li	a1, 0x80
   111d4: 00010637     	lui	a2, 0x10
   111d8: 08b105a3     	sb	a1, 0x8b(sp)
   111dc: 0c012223     	sw	zero, 0xc4(sp)
   111e0: 0d212423     	sw	s2, 0xc8(sp)
   111e4: 0ca10623     	sb	a0, 0xcc(sp)
   111e8: 08012503     	lw	a0, 0x80(sp)
   111ec: f0060413     	addi	s0, a2, -0x100
   111f0: 08412583     	lw	a1, 0x84(sp)
   111f4: 08812603     	lw	a2, 0x88(sp)
   111f8: 00855693     	srli	a3, a0, 0x8
   111fc: 01855713     	srli	a4, a0, 0x18
   11200: 008577b3     	and	a5, a0, s0
   11204: 01851513     	slli	a0, a0, 0x18
   11208: 0086f6b3     	and	a3, a3, s0
   1120c: 00e6e6b3     	or	a3, a3, a4
   11210: 0085d713     	srli	a4, a1, 0x8
   11214: 00879793     	slli	a5, a5, 0x8
   11218: 00f56533     	or	a0, a0, a5
   1121c: 0185d793     	srli	a5, a1, 0x18
   11220: 00877733     	and	a4, a4, s0
   11224: 00f76733     	or	a4, a4, a5
   11228: 0085f7b3     	and	a5, a1, s0
   1122c: 01859593     	slli	a1, a1, 0x18
   11230: 00879793     	slli	a5, a5, 0x8
   11234: 00f5e5b3     	or	a1, a1, a5
   11238: 00d56533     	or	a0, a0, a3
   1123c: 00865693     	srli	a3, a2, 0x8
   11240: 00e5e5b3     	or	a1, a1, a4
   11244: 01865713     	srli	a4, a2, 0x18
   11248: 0086f6b3     	and	a3, a3, s0
   1124c: 00e6e6b3     	or	a3, a3, a4
   11250: 00867733     	and	a4, a2, s0
   11254: 01861613     	slli	a2, a2, 0x18
   11258: 00871713     	slli	a4, a4, 0x8
   1125c: 00e66633     	or	a2, a2, a4
   11260: 00d66633     	or	a2, a2, a3
   11264: 08010993     	addi	s3, sp, 0x80
   11268: 08a12023     	sw	a0, 0x80(sp)
   1126c: 08b12223     	sw	a1, 0x84(sp)
   11270: 08c12423     	sw	a2, 0x88(sp)
   11274: 0d212023     	sw	s2, 0xc0(sp)
   11278: 08c10513     	addi	a0, sp, 0x8c
   1127c: 03000613     	li	a2, 0x30
   11280: 00000593     	li	a1, 0x0
   11284: 00000097     	auipc	ra, 0x0
   11288: 3c0080e7     	jalr	0x3c0(ra) <memset>
   1128c: 05800513     	li	a0, 0x58
   11290: 0aa12e23     	sw	a0, 0xbc(sp)
   11294: 0099900b     	<unknown>
   11298: 07012803     	lw	a6, 0x70(sp)
   1129c: 07412603     	lw	a2, 0x74(sp)
   112a0: 07812583     	lw	a1, 0x78(sp)
   112a4: 07c12503     	lw	a0, 0x7c(sp)
   112a8: 06012683     	lw	a3, 0x60(sp)
   112ac: 06412703     	lw	a4, 0x64(sp)
   112b0: 06812783     	lw	a5, 0x68(sp)
   112b4: 06c12883     	lw	a7, 0x6c(sp)
   112b8: 0086d293     	srli	t0, a3, 0x8
   112bc: 0186d313     	srli	t1, a3, 0x18
   112c0: 0086f3b3     	and	t2, a3, s0
   112c4: 01869e13     	slli	t3, a3, 0x18
   112c8: 00875e93     	srli	t4, a4, 0x8
   112cc: 01875f13     	srli	t5, a4, 0x18
   112d0: 00877fb3     	and	t6, a4, s0
   112d4: 01871493     	slli	s1, a4, 0x18
   112d8: 0087d913     	srli	s2, a5, 0x8
   112dc: 0187d993     	srli	s3, a5, 0x18
   112e0: 0087fa33     	and	s4, a5, s0
   112e4: 01879a93     	slli	s5, a5, 0x18
   112e8: 0082f6b3     	and	a3, t0, s0
   112ec: 0066e6b3     	or	a3, a3, t1
   112f0: 0088d313     	srli	t1, a7, 0x8
   112f4: 00839393     	slli	t2, t2, 0x8
   112f8: 007e6733     	or	a4, t3, t2
   112fc: 0188d393     	srli	t2, a7, 0x18
   11300: 008ef7b3     	and	a5, t4, s0
   11304: 01e7e7b3     	or	a5, a5, t5
   11308: 0088fe33     	and	t3, a7, s0
   1130c: 01889e93     	slli	t4, a7, 0x18
   11310: 008f9f93     	slli	t6, t6, 0x8
   11314: 01f4e8b3     	or	a7, s1, t6
   11318: 00885f13     	srli	t5, a6, 0x8
   1131c: 008972b3     	and	t0, s2, s0
   11320: 0132e2b3     	or	t0, t0, s3
   11324: 01885f93     	srli	t6, a6, 0x18
   11328: 008a1a13     	slli	s4, s4, 0x8
   1132c: 014ae4b3     	or	s1, s5, s4
   11330: 00887933     	and	s2, a6, s0
   11334: 01881813     	slli	a6, a6, 0x18
   11338: 00837333     	and	t1, t1, s0
   1133c: 00736333     	or	t1, t1, t2
   11340: 00865393     	srli	t2, a2, 0x8
   11344: 008e1e13     	slli	t3, t3, 0x8
   11348: 01ceee33     	or	t3, t4, t3
   1134c: 01865e93     	srli	t4, a2, 0x18
   11350: 008f7f33     	and	t5, t5, s0
   11354: 01ff6f33     	or	t5, t5, t6
   11358: 00867fb3     	and	t6, a2, s0
   1135c: 01861613     	slli	a2, a2, 0x18
   11360: 00891913     	slli	s2, s2, 0x8
   11364: 01286833     	or	a6, a6, s2
   11368: 0085d913     	srli	s2, a1, 0x8
   1136c: 0083f3b3     	and	t2, t2, s0
   11370: 01d3e3b3     	or	t2, t2, t4
   11374: 0185de93     	srli	t4, a1, 0x18
   11378: 008f9f93     	slli	t6, t6, 0x8
   1137c: 01f66633     	or	a2, a2, t6
   11380: 0085ffb3     	and	t6, a1, s0
   11384: 01859593     	slli	a1, a1, 0x18
   11388: 00897933     	and	s2, s2, s0
   1138c: 01d96eb3     	or	t4, s2, t4
   11390: 00855913     	srli	s2, a0, 0x8
   11394: 008f9f93     	slli	t6, t6, 0x8
   11398: 01f5e5b3     	or	a1, a1, t6
   1139c: 01855f93     	srli	t6, a0, 0x18
   113a0: 00897933     	and	s2, s2, s0
   113a4: 01f96fb3     	or	t6, s2, t6
   113a8: 00857433     	and	s0, a0, s0
   113ac: 01851513     	slli	a0, a0, 0x18
   113b0: 00841413     	slli	s0, s0, 0x8
   113b4: 00856533     	or	a0, a0, s0
   113b8: 00d766b3     	or	a3, a4, a3
   113bc: 00f8e733     	or	a4, a7, a5
   113c0: 0054e7b3     	or	a5, s1, t0
   113c4: 006e68b3     	or	a7, t3, t1
   113c8: 01e86833     	or	a6, a6, t5
   113cc: 00766633     	or	a2, a2, t2
   113d0: 01d5e5b3     	or	a1, a1, t4
   113d4: 01f56533     	or	a0, a0, t6
   113d8: 06d12023     	sw	a3, 0x60(sp)
   113dc: 06e12223     	sw	a4, 0x64(sp)
   113e0: 06f12423     	sw	a5, 0x68(sp)
   113e4: 07112623     	sw	a7, 0x6c(sp)
   113e8: 07012823     	sw	a6, 0x70(sp)
   113ec: 06c12a23     	sw	a2, 0x74(sp)
   113f0: 06b12c23     	sw	a1, 0x78(sp)
   113f4: 06a12e23     	sw	a0, 0x7c(sp)
   113f8: 06010513     	addi	a0, sp, 0x60
   113fc: 0000006f     	j	0x113fc <_start+0x284>

00011400 <memcpy>:
   11400: ff010113     	addi	sp, sp, -0x10
   11404: 00112623     	sw	ra, 0xc(sp)
   11408: 00812423     	sw	s0, 0x8(sp)
   1140c: 01010413     	addi	s0, sp, 0x10
   11410: 00c12083     	lw	ra, 0xc(sp)
   11414: 00812403     	lw	s0, 0x8(sp)
   11418: 01010113     	addi	sp, sp, 0x10
   1141c: 00000317     	auipc	t1, 0x0
   11420: 00830067     	jr	0x8(t1) <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c>

00011424 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c>:
   11424: fe010113     	addi	sp, sp, -0x20
   11428: 00112e23     	sw	ra, 0x1c(sp)
   1142c: 00812c23     	sw	s0, 0x18(sp)
   11430: 02010413     	addi	s0, sp, 0x20
   11434: 01000693     	li	a3, 0x10
   11438: 08d66063     	bltu	a2, a3, 0x114b8 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x94>
   1143c: 40a006b3     	neg	a3, a0
   11440: 0036f693     	andi	a3, a3, 0x3
   11444: 00d507b3     	add	a5, a0, a3
   11448: 02f57463     	bgeu	a0, a5, 0x11470 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x4c>
   1144c: 00068713     	mv	a4, a3
   11450: 00050813     	mv	a6, a0
   11454: 00058893     	mv	a7, a1
   11458: 0008c283     	lbu	t0, 0x0(a7)
   1145c: fff70713     	addi	a4, a4, -0x1
   11460: 00580023     	sb	t0, 0x0(a6)
   11464: 00180813     	addi	a6, a6, 0x1
   11468: 00188893     	addi	a7, a7, 0x1
   1146c: fe0716e3     	bnez	a4, 0x11458 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x34>
   11470: 00d585b3     	add	a1, a1, a3
   11474: 40d60633     	sub	a2, a2, a3
   11478: ffc67713     	andi	a4, a2, -0x4
   1147c: 0035f893     	andi	a7, a1, 0x3
   11480: 00e786b3     	add	a3, a5, a4
   11484: 06089463     	bnez	a7, 0x114ec <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xc8>
   11488: 00d7fe63     	bgeu	a5, a3, 0x114a4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x80>
   1148c: 00058813     	mv	a6, a1
   11490: 00082883     	lw	a7, 0x0(a6)
   11494: 0117a023     	sw	a7, 0x0(a5)
   11498: 00478793     	addi	a5, a5, 0x4
   1149c: 00480813     	addi	a6, a6, 0x4
   114a0: fed7e8e3     	bltu	a5, a3, 0x11490 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x6c>
   114a4: 00e585b3     	add	a1, a1, a4
   114a8: 00367613     	andi	a2, a2, 0x3
   114ac: 00c68733     	add	a4, a3, a2
   114b0: 00e6ea63     	bltu	a3, a4, 0x114c4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   114b4: 0280006f     	j	0x114dc <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>
   114b8: 00050693     	mv	a3, a0
   114bc: 00c50733     	add	a4, a0, a2
   114c0: 00e57e63     	bgeu	a0, a4, 0x114dc <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>
   114c4: 0005c703     	lbu	a4, 0x0(a1)
   114c8: fff60613     	addi	a2, a2, -0x1
   114cc: 00e68023     	sb	a4, 0x0(a3)
   114d0: 00168693     	addi	a3, a3, 0x1
   114d4: 00158593     	addi	a1, a1, 0x1
   114d8: fe0616e3     	bnez	a2, 0x114c4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   114dc: 01c12083     	lw	ra, 0x1c(sp)
   114e0: 01812403     	lw	s0, 0x18(sp)
   114e4: 02010113     	addi	sp, sp, 0x20
   114e8: 00008067     	ret
   114ec: 00000813     	li	a6, 0x0
   114f0: 00400293     	li	t0, 0x4
   114f4: fe042a23     	sw	zero, -0xc(s0)
   114f8: 41128333     	sub	t1, t0, a7
   114fc: ff440293     	addi	t0, s0, -0xc
   11500: 00137393     	andi	t2, t1, 0x1
   11504: 0112e2b3     	or	t0, t0, a7
   11508: 04039e63     	bnez	t2, 0x11564 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x140>
   1150c: 00237313     	andi	t1, t1, 0x2
   11510: 06031463     	bnez	t1, 0x11578 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x154>
   11514: ff442e83     	lw	t4, -0xc(s0)
   11518: 00389813     	slli	a6, a7, 0x3
   1151c: 00478293     	addi	t0, a5, 0x4
   11520: 41158f33     	sub	t5, a1, a7
   11524: 06d2fc63     	bgeu	t0, a3, 0x1159c <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x178>
   11528: 410002b3     	neg	t0, a6
   1152c: 0182fe13     	andi	t3, t0, 0x18
   11530: 004f2283     	lw	t0, 0x4(t5)
   11534: 004f0393     	addi	t2, t5, 0x4
   11538: 010edeb3     	srl	t4, t4, a6
   1153c: 00478313     	addi	t1, a5, 0x4
   11540: 01c29f33     	sll	t5, t0, t3
   11544: 01df6eb3     	or	t4, t5, t4
   11548: 00878f93     	addi	t6, a5, 0x8
   1154c: 01d7a023     	sw	t4, 0x0(a5)
   11550: 00030793     	mv	a5, t1
   11554: 00038f13     	mv	t5, t2
   11558: 00028e93     	mv	t4, t0
   1155c: fcdfeae3     	bltu	t6, a3, 0x11530 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x10c>
   11560: 0480006f     	j	0x115a8 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x184>
   11564: 0005c803     	lbu	a6, 0x0(a1)
   11568: 01028023     	sb	a6, 0x0(t0)
   1156c: 00100813     	li	a6, 0x1
   11570: 00237313     	andi	t1, t1, 0x2
   11574: fa0300e3     	beqz	t1, 0x11514 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xf0>
   11578: 01058333     	add	t1, a1, a6
   1157c: 00031303     	lh	t1, 0x0(t1)
   11580: 01028833     	add	a6, t0, a6
   11584: 00681023     	sh	t1, 0x0(a6)
   11588: ff442e83     	lw	t4, -0xc(s0)
   1158c: 00389813     	slli	a6, a7, 0x3
   11590: 00478293     	addi	t0, a5, 0x4
   11594: 41158f33     	sub	t5, a1, a7
   11598: f8d2e8e3     	bltu	t0, a3, 0x11528 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x104>
   1159c: 000e8293     	mv	t0, t4
   115a0: 000f0393     	mv	t2, t5
   115a4: 00078313     	mv	t1, a5
   115a8: fe040823     	sb	zero, -0x10(s0)
   115ac: 00100793     	li	a5, 0x1
   115b0: fe040723     	sb	zero, -0x12(s0)
   115b4: 00f89c63     	bne	a7, a5, 0x115cc <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1a8>
   115b8: 00000893     	li	a7, 0x0
   115bc: 00000793     	li	a5, 0x0
   115c0: 00000e13     	li	t3, 0x0
   115c4: ff040e93     	addi	t4, s0, -0x10
   115c8: 01c0006f     	j	0x115e4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1c0>
   115cc: 0043c883     	lbu	a7, 0x4(t2)
   115d0: 0053c783     	lbu	a5, 0x5(t2)
   115d4: 00200e13     	li	t3, 0x2
   115d8: ff140823     	sb	a7, -0x10(s0)
   115dc: 00879793     	slli	a5, a5, 0x8
   115e0: fee40e93     	addi	t4, s0, -0x12
   115e4: 0015ff13     	andi	t5, a1, 0x1
   115e8: 000f1663     	bnez	t5, 0x115f4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1d0>
   115ec: 00000393     	li	t2, 0x0
   115f0: 0200006f     	j	0x11610 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1ec>
   115f4: 00438393     	addi	t2, t2, 0x4
   115f8: 01c383b3     	add	t2, t2, t3
   115fc: 0003c883     	lbu	a7, 0x0(t2)
   11600: 011e8023     	sb	a7, 0x0(t4)
   11604: fee44383     	lbu	t2, -0x12(s0)
   11608: ff044883     	lbu	a7, -0x10(s0)
   1160c: 01039393     	slli	t2, t2, 0x10
   11610: 0113e8b3     	or	a7, t2, a7
   11614: 0102d2b3     	srl	t0, t0, a6
   11618: 41000833     	neg	a6, a6
   1161c: 0117e7b3     	or	a5, a5, a7
   11620: 01887813     	andi	a6, a6, 0x18
   11624: 010797b3     	sll	a5, a5, a6
   11628: 0057e7b3     	or	a5, a5, t0
   1162c: 00f32023     	sw	a5, 0x0(t1)
   11630: 00e585b3     	add	a1, a1, a4
   11634: 00367613     	andi	a2, a2, 0x3
   11638: 00c68733     	add	a4, a3, a2
   1163c: e8e6e4e3     	bltu	a3, a4, 0x114c4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   11640: e9dff06f     	j	0x114dc <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>

00011644 <memset>:
   11644: ff010113     	addi	sp, sp, -0x10
   11648: 00112623     	sw	ra, 0xc(sp)
   1164c: 00812423     	sw	s0, 0x8(sp)
   11650: 01010413     	addi	s0, sp, 0x10
   11654: 01000693     	li	a3, 0x10
   11658: 08d66263     	bltu	a2, a3, 0x116dc <memset+0x98>
   1165c: 40a006b3     	neg	a3, a0
   11660: 0036f693     	andi	a3, a3, 0x3
   11664: 00d50733     	add	a4, a0, a3
   11668: 00e57e63     	bgeu	a0, a4, 0x11684 <memset+0x40>
   1166c: 00068793     	mv	a5, a3
   11670: 00050813     	mv	a6, a0
   11674: 00b80023     	sb	a1, 0x0(a6)
   11678: fff78793     	addi	a5, a5, -0x1
   1167c: 00180813     	addi	a6, a6, 0x1
   11680: fe079ae3     	bnez	a5, 0x11674 <memset+0x30>
   11684: 40d60633     	sub	a2, a2, a3
   11688: ffc67693     	andi	a3, a2, -0x4
   1168c: 00d706b3     	add	a3, a4, a3
   11690: 02d77063     	bgeu	a4, a3, 0x116b0 <memset+0x6c>
   11694: 0ff5f793     	andi	a5, a1, 0xff
   11698: 01010837     	lui	a6, 0x1010
   1169c: 10180813     	addi	a6, a6, 0x101
   116a0: 030787b3     	mul	a5, a5, a6
   116a4: 00f72023     	sw	a5, 0x0(a4)
   116a8: 00470713     	addi	a4, a4, 0x4
   116ac: fed76ce3     	bltu	a4, a3, 0x116a4 <memset+0x60>
   116b0: 00367613     	andi	a2, a2, 0x3
   116b4: 00c68733     	add	a4, a3, a2
   116b8: 00e6fa63     	bgeu	a3, a4, 0x116cc <memset+0x88>
   116bc: 00b68023     	sb	a1, 0x0(a3)
   116c0: fff60613     	addi	a2, a2, -0x1
   116c4: 00168693     	addi	a3, a3, 0x1
   116c8: fe061ae3     	bnez	a2, 0x116bc <memset+0x78>
   116cc: 00c12083     	lw	ra, 0xc(sp)
   116d0: 00812403     	lw	s0, 0x8(sp)
   116d4: 01010113     	addi	sp, sp, 0x10
   116d8: 00008067     	ret
   116dc: 00050693     	mv	a3, a0
   116e0: 00c50733     	add	a4, a0, a2
   116e4: fce56ce3     	bltu	a0, a4, 0x116bc <memset+0x78>
   116e8: fe5ff06f     	j	0x116cc <memset+0x88>
