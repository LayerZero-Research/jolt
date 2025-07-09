
inline_benchmark:	file format elf32-littleriscv

Disassembly of section .text:

000112f8 <_start>:
   112f8:      	addi	sp, sp, -0x1e0
   112fc:      	sw	ra, 0x1dc(sp)
   11300:      	sw	s0, 0x1d8(sp)
   11304:      	sw	s1, 0x1d4(sp)
   11308:      	sw	s2, 0x1d0(sp)
   1130c:      	sw	s3, 0x1cc(sp)
   11310:      	sw	s4, 0x1c8(sp)
   11314:      	sw	s5, 0x1c4(sp)
   11318:      	sw	s6, 0x1c0(sp)
   1131c:      	sw	s7, 0x1bc(sp)
   11320:      	sw	s8, 0x1b8(sp)
   11324:      	sw	s9, 0x1b4(sp)
   11328:      	sw	s10, 0x1b0(sp)
   1132c:      	sw	s11, 0x1ac(sp)
   11330:      	addi	a0, sp, 0x28
   11334:      	li	a2, 0xc8
   11338:      	li	a1, 0x0
   1133c:      	auipc	ra, 0x1
   11340:      	jalr	0x37c(ra) <memset>
   11344:      	addi	s0, sp, 0xf8
   11348:      	li	a2, 0x89
   1134c:      	mv	a0, s0
   11350:      	li	a1, 0x0
   11354:      	auipc	ra, 0x1
   11358:      	jalr	0x364(ra) <memset>
   1135c:      	li	a0, 0x18
   11360:      	sw	a0, 0xf0(sp)
   11364:      	lui	a1, 0x10
   11368:      	addi	a1, a1, 0xd8
   1136c:      	li	a2, 0xb
   11370:      	li	s1, 0xb
   11374:      	mv	a0, s0
   11378:      	auipc	ra, 0x1
   1137c:      	jalr	0xfc(ra) <memcpy>
   11380:      	sb	s1, 0x180(sp)
   11384:      	addi	a0, sp, 0x103
   11388:      	li	a2, 0x7d
   1138c:      	li	a1, 0x0
   11390:      	auipc	ra, 0x1
   11394:      	jalr	0x328(ra) <memset>
   11398:      	li	a1, 0x6
   1139c:      	lw	a3, 0xf8(sp)
   113a0:      	lw	a2, 0xfc(sp)
   113a4:      	lw	a4, 0x28(sp)
   113a8:      	lw	a6, 0x2c(sp)
   113ac:      	lw	a0, 0x30(sp)
   113b0:      	sw	a0, 0x24(sp)
   113b4:      	lw	t0, 0x34(sp)
   113b8:      	lw	t3, 0x104(sp)
   113bc:      	lw	a7, 0x108(sp)
   113c0:      	lw	t4, 0x10c(sp)
   113c4:      	lw	t5, 0x110(sp)
   113c8:      	lw	t6, 0x38(sp)
   113cc:      	lw	s0, 0x3c(sp)
   113d0:      	lw	s1, 0x40(sp)
   113d4:      	lw	s2, 0x44(sp)
   113d8:      	lw	s3, 0x114(sp)
   113dc:      	lw	t1, 0x118(sp)
   113e0:      	lw	t2, 0x11c(sp)
   113e4:      	lw	s4, 0x120(sp)
   113e8:      	lw	s5, 0x48(sp)
   113ec:      	lw	s6, 0x4c(sp)
   113f0:      	lw	s7, 0x50(sp)
   113f4:      	lw	s8, 0x54(sp)
   113f8:      	lw	s9, 0x124(sp)
   113fc:      	lw	s10, 0x128(sp)
   11400:      	lw	s11, 0x12c(sp)
   11404:      	lw	ra, 0x130(sp)
   11408:      	sb	a1, 0x103(sp)
   1140c:      	lw	a0, 0x100(sp)
   11410:      	sw	a0, 0x14(sp)
   11414:      	xor	a0, a6, a2
   11418:      	sw	a0, 0x20(sp)
   1141c:      	xor	a3, a4, a3
   11420:      	sw	a3, 0x18(sp)
   11424:      	xor	a0, t0, t3
   11428:      	sw	a0, 0x1c(sp)
   1142c:      	lw	a2, 0x58(sp)
   11430:      	lw	t3, 0x5c(sp)
   11434:      	lw	a1, 0x60(sp)
   11438:      	lw	a0, 0x64(sp)
   1143c:      	xor	a3, s0, t4
   11440:      	sw	a3, 0x10(sp)
   11444:      	xor	a3, t6, a7
   11448:      	sw	a3, 0x8(sp)
   1144c:      	xor	a3, s2, s3
   11450:      	sw	a3, 0xc(sp)
   11454:      	xor	a3, s1, t5
   11458:      	sw	a3, 0x4(sp)
   1145c:      	lw	t4, 0x134(sp)
   11460:      	lw	s2, 0x138(sp)
   11464:      	lw	s3, 0x13c(sp)
   11468:      	lw	t5, 0x140(sp)
   1146c:      	xor	s0, s6, t2
   11470:      	xor	s1, s5, t1
   11474:      	xor	t1, s7, s4
   11478:      	xor	t2, s8, s9
   1147c:      	lw	s4, 0x68(sp)
   11480:      	lw	s5, 0x6c(sp)
   11484:      	lw	s6, 0x70(sp)
   11488:      	lw	s7, 0x74(sp)
   1148c:      	xor	t3, t3, s11
   11490:      	xor	t0, a2, s10
   11494:      	xor	t4, a0, t4
   11498:      	xor	t6, a1, ra
   1149c:      	lw	a0, 0x144(sp)
   114a0:      	lw	a1, 0x148(sp)
   114a4:      	lw	s8, 0x14c(sp)
   114a8:      	lw	s9, 0x150(sp)
   114ac:      	xor	s2, s4, s2
   114b0:      	xor	s3, s5, s3
   114b4:      	xor	s4, s6, t5
   114b8:      	xor	s5, s7, a0
   114bc:      	lw	a0, 0x78(sp)
   114c0:      	lw	t5, 0x7c(sp)
   114c4:      	lw	s10, 0x80(sp)
   114c8:      	lw	s11, 0x84(sp)
   114cc:      	xor	s6, a0, a1
   114d0:      	xor	s7, t5, s8
   114d4:      	xor	s8, s10, s9
   114d8:      	lw	a0, 0x154(sp)
   114dc:      	lw	a1, 0x158(sp)
   114e0:      	lw	t5, 0x15c(sp)
   114e4:      	lw	ra, 0x160(sp)
   114e8:      	xor	s9, s11, a0
   114ec:      	lw	a0, 0x88(sp)
   114f0:      	lw	s11, 0x8c(sp)
   114f4:      	lw	s10, 0x90(sp)
   114f8:      	lw	a2, 0x94(sp)
   114fc:      	xor	a7, a0, a1
   11500:      	xor	s11, s11, t5
   11504:      	xor	ra, s10, ra
   11508:      	lw	a1, 0x164(sp)
   1150c:      	lw	t5, 0x168(sp)
   11510:      	lw	s10, 0x16c(sp)
   11514:      	lw	a4, 0x170(sp)
   11518:      	xor	a6, a2, a1
   1151c:      	lbu	a3, 0x17f(sp)
   11520:      	lw	a2, 0x98(sp)
   11524:      	lw	a1, 0x9c(sp)
   11528:      	lw	a0, 0xa0(sp)
   1152c:      	lw	a5, 0xa4(sp)
   11530:      	xor	a2, a2, t5
   11534:      	xor	t5, a1, s10
   11538:      	xor	a0, a0, a4
   1153c:      	ori	a1, a3, 0x80
   11540:      	sb	a1, 0x17f(sp)
   11544:      	lw	a1, 0x8(sp)
   11548:      	sw	a1, 0x38(sp)
   1154c:      	lw	a1, 0x10(sp)
   11550:      	sw	a1, 0x3c(sp)
   11554:      	lw	a1, 0x4(sp)
   11558:      	sw	a1, 0x40(sp)
   1155c:      	lw	a1, 0xc(sp)
   11560:      	sw	a1, 0x44(sp)
   11564:      	sw	s1, 0x48(sp)
   11568:      	sw	s0, 0x4c(sp)
   1156c:      	lw	a1, 0xf0(sp)
   11570:      	sw	t1, 0x50(sp)
   11574:      	sw	t2, 0x54(sp)
   11578:      	sw	t0, 0x58(sp)
   1157c:      	sw	t3, 0x5c(sp)
   11580:      	sw	t6, 0x60(sp)
   11584:      	sw	t4, 0x64(sp)
   11588:      	sw	s2, 0x68(sp)
   1158c:      	sw	s3, 0x6c(sp)
   11590:      	sw	s4, 0x70(sp)
   11594:      	sw	s5, 0x74(sp)
   11598:      	sw	s6, 0x78(sp)
   1159c:      	sw	s7, 0x7c(sp)
   115a0:      	sw	s8, 0x80(sp)
   115a4:      	sw	s9, 0x84(sp)
   115a8:      	lw	a3, 0xa8(sp)
   115ac:      	sw	a7, 0x88(sp)
   115b0:      	sw	s11, 0x8c(sp)
   115b4:      	sw	ra, 0x90(sp)
   115b8:      	sw	a6, 0x94(sp)
   115bc:      	lw	a4, 0x174(sp)
   115c0:      	lw	a6, 0x24(sp)
   115c4:      	lw	a7, 0x14(sp)
   115c8:      	xor	a6, a6, a7
   115cc:      	lw	a7, 0x178(sp)
   115d0:      	lw	t0, 0x17c(sp)
   115d4:      	xor	a4, a5, a4
   115d8:      	lw	a5, 0xac(sp)
   115dc:      	xor	a3, a3, a7
   115e0:      	lw	a7, 0x18(sp)
   115e4:      	sw	a7, 0x28(sp)
   115e8:      	lw	a7, 0x20(sp)
   115ec:      	sw	a7, 0x2c(sp)
   115f0:      	sw	a6, 0x30(sp)
   115f4:      	lw	a6, 0x1c(sp)
   115f8:      	sw	a6, 0x34(sp)
   115fc:      	sw	a2, 0x98(sp)
   11600:      	sw	t5, 0x9c(sp)
   11604:      	sw	a0, 0xa0(sp)
   11608:      	sw	a4, 0xa4(sp)
   1160c:      	sb	zero, 0x180(sp)
   11610:      	xor	a0, a5, t0
   11614:      	sw	a3, 0xa8(sp)
   11618:      	sw	a0, 0xac(sp)
   1161c:      	addi	a0, sp, 0x28
   11620:      	auipc	ra, 0x0
   11624:      	jalr	0x7c(ra) <keccak::p1600::he90e6ecc89f09623>
   11628:      	lw	a0, 0x38(sp)
   1162c:      	lw	a1, 0x3c(sp)
   11630:      	lw	a2, 0x40(sp)
   11634:      	lw	a3, 0x44(sp)
   11638:      	sw	a0, 0x19c(sp)
   1163c:      	sw	a1, 0x1a0(sp)
   11640:      	sw	a2, 0x1a4(sp)
   11644:      	sw	a3, 0x1a8(sp)
   11648:      	lw	a0, 0x28(sp)
   1164c:      	lw	a1, 0x2c(sp)
   11650:      	lw	a2, 0x30(sp)
   11654:      	lw	a3, 0x34(sp)
   11658:      	sw	a0, 0x18c(sp)
   1165c:      	sw	a1, 0x190(sp)
   11660:      	sw	a2, 0x194(sp)
   11664:      	sw	a3, 0x198(sp)
   11668:      	addi	a0, sp, 0x18c
   1166c:      	j	0x1166c <_start+0x374>

00011670 <core::panicking::panic_fmt::hcd8c154b4dd14503>:
   11670:      	addi	sp, sp, -0x10
   11674:      	sw	ra, 0xc(sp)
   11678:      	sw	s0, 0x8(sp)
   1167c:      	addi	s0, sp, 0x10
   11680:      	j	0x11680 <core::panicking::panic_fmt::hcd8c154b4dd14503+0x10>

00011684 <core::panicking::panic::h8e943910cbb9fd4a>:
   11684:      	addi	sp, sp, -0x10
   11688:      	sw	ra, 0xc(sp)
   1168c:      	sw	s0, 0x8(sp)
   11690:      	addi	s0, sp, 0x10
   11694:      	auipc	ra, 0x0
   11698:      	jalr	-0x24(ra) <core::panicking::panic_fmt::hcd8c154b4dd14503>

0001169c <keccak::p1600::he90e6ecc89f09623>:
   1169c:      	addi	sp, sp, -0x110
   116a0:      	sw	ra, 0x10c(sp)
   116a4:      	sw	s0, 0x108(sp)
   116a8:      	sw	s1, 0x104(sp)
   116ac:      	sw	s2, 0x100(sp)
   116b0:      	sw	s3, 0xfc(sp)
   116b4:      	sw	s4, 0xf8(sp)
   116b8:      	sw	s5, 0xf4(sp)
   116bc:      	sw	s6, 0xf0(sp)
   116c0:      	sw	s7, 0xec(sp)
   116c4:      	sw	s8, 0xe8(sp)
   116c8:      	sw	s9, 0xe4(sp)
   116cc:      	sw	s10, 0xe0(sp)
   116d0:      	sw	s11, 0xdc(sp)
   116d4:      	li	a2, 0x18
   116d8:      	bltu	a2, a1, 0x1246c <keccak::p1600::he90e6ecc89f09623+0xdd0>
   116dc:      	beqz	a1, 0x12430 <keccak::p1600::he90e6ecc89f09623+0xd94>
   116e0:      	slli	a7, a1, 0x3
   116e4:      	lw	a1, 0x20(a0)
   116e8:      	sw	a1, 0xd4(sp)
   116ec:      	lw	a1, 0x24(a0)
   116f0:      	sw	a1, 0x6c(sp)
   116f4:      	lw	a1, 0x28(a0)
   116f8:      	sw	a1, 0xb4(sp)
   116fc:      	lw	a1, 0x2c(a0)
   11700:      	sw	a1, 0xb8(sp)
   11704:      	lw	a1, 0x70(a0)
   11708:      	sw	a1, 0xa8(sp)
   1170c:      	lw	a1, 0x74(a0)
   11710:      	sw	a1, 0xd0(sp)
   11714:      	lw	t5, 0x78(a0)
   11718:      	lw	s6, 0x7c(a0)
   1171c:      	lw	a6, 0x0(a0)
   11720:      	lw	a5, 0x4(a0)
   11724:      	lw	a1, 0x8(a0)
   11728:      	sw	a1, 0x68(sp)
   1172c:      	lw	a1, 0xc(a0)
   11730:      	sw	a1, 0x64(sp)
   11734:      	lw	ra, 0x50(a0)
   11738:      	lw	t0, 0x54(a0)
   1173c:      	lw	s7, 0x58(a0)
   11740:      	lw	a1, 0x5c(a0)
   11744:      	sw	a1, 0x78(sp)
   11748:      	lw	a1, 0xa0(a0)
   1174c:      	sw	a1, 0x54(sp)
   11750:      	lw	a1, 0xa4(a0)
   11754:      	sw	a1, 0x58(sp)
   11758:      	lw	s0, 0xa8(a0)
   1175c:      	lw	s10, 0xac(a0)
   11760:      	lw	t1, 0x30(a0)
   11764:      	lw	a1, 0x34(a0)
   11768:      	sw	a1, 0xb0(sp)
   1176c:      	lw	a1, 0x38(a0)
   11770:      	sw	a1, 0x90(sp)
   11774:      	lw	a1, 0x3c(a0)
   11778:      	sw	a1, 0xac(sp)
   1177c:      	lw	s11, 0x80(a0)
   11780:      	lw	s5, 0x84(a0)
   11784:      	lw	s9, 0x88(a0)
   11788:      	lw	t4, 0x8c(a0)
   1178c:      	lw	a1, 0x10(a0)
   11790:      	sw	a1, 0xc0(sp)
   11794:      	lw	a1, 0x14(a0)
   11798:      	sw	a1, 0xc8(sp)
   1179c:      	lw	a1, 0x18(a0)
   117a0:      	sw	a1, 0xc4(sp)
   117a4:      	lw	a1, 0x1c(a0)
   117a8:      	sw	a1, 0xcc(sp)
   117ac:      	lw	s1, 0x60(a0)
   117b0:      	lw	a1, 0x64(a0)
   117b4:      	sw	a1, 0x70(sp)
   117b8:      	lw	a1, 0x68(a0)
   117bc:      	sw	a1, 0x74(sp)
   117c0:      	lw	a1, 0x6c(a0)
   117c4:      	sw	a1, 0x8c(sp)
   117c8:      	lw	a1, 0xb0(a0)
   117cc:      	sw	a1, 0x5c(sp)
   117d0:      	lw	a1, 0xb4(a0)
   117d4:      	sw	a1, 0xa0(sp)
   117d8:      	lw	a1, 0xb8(a0)
   117dc:      	sw	a1, 0x60(sp)
   117e0:      	lw	a1, 0xbc(a0)
   117e4:      	sw	a1, 0x80(sp)
   117e8:      	lw	a1, 0x40(a0)
   117ec:      	sw	a1, 0x94(sp)
   117f0:      	lw	a1, 0x44(a0)
   117f4:      	sw	a1, 0xd8(sp)
   117f8:      	lw	a1, 0x48(a0)
   117fc:      	sw	a1, 0x98(sp)
   11800:      	lw	a1, 0x4c(a0)
   11804:      	sw	a1, 0xbc(sp)
   11808:      	lw	a1, 0x90(a0)
   1180c:      	sw	a1, 0x9c(sp)
   11810:      	lw	a1, 0x94(a0)
   11814:      	sw	a1, 0x7c(sp)
   11818:      	lw	a1, 0x98(a0)
   1181c:      	sw	a1, 0xa4(sp)
   11820:      	lw	a1, 0x9c(a0)
   11824:      	sw	a1, 0x84(sp)
   11828:      	lw	a1, 0xc0(a0)
   1182c:      	sw	a1, 0x88(sp)
   11830:      	sw	a0, 0x8(sp)
   11834:      	lw	a3, 0xc4(a0)
   11838:      	lui	a2, 0x10
   1183c:      	addi	a2, a2, 0xe8
   11840:      	sub	a4, a2, a7
   11844:      	mv	a7, t0
   11848:      	addi	s8, a4, 0xc0
   1184c:      	addi	a0, a2, 0xc0
   11850:      	sw	a0, 0xc(sp)
   11854:      	sw	s6, 0x30(sp)
   11858:      	sw	a6, 0x1c(sp)
   1185c:      	sw	t5, 0x2c(sp)
   11860:      	sw	a5, 0x44(sp)
   11864:      	sw	a7, 0x34(sp)
   11868:      	sw	t1, 0x40(sp)
   1186c:      	sw	s8, 0x50(sp)
   11870:      	lw	a2, 0xb8(sp)
   11874:      	xor	a2, a2, a5
   11878:      	xor	a4, s6, a7
   1187c:      	xor	a7, a4, a2
   11880:      	lw	a0, 0xb4(sp)
   11884:      	xor	a2, a0, a6
   11888:      	sw	ra, 0x3c(sp)
   1188c:      	xor	a4, t5, ra
   11890:      	xor	t0, a4, a2
   11894:      	lw	s2, 0x68(sp)
   11898:      	xor	a2, t1, s2
   1189c:      	sw	s7, 0x48(sp)
   118a0:      	sw	s11, 0x4c(sp)
   118a4:      	xor	a4, s11, s7
   118a8:      	xor	a2, a4, a2
   118ac:      	lw	a0, 0x64(sp)
   118b0:      	lw	a4, 0xb0(sp)
   118b4:      	xor	a4, a4, a0
   118b8:      	sw	s5, 0x20(sp)
   118bc:      	lw	a5, 0x78(sp)
   118c0:      	xor	a6, s5, a5
   118c4:      	xor	a4, a6, a4
   118c8:      	lw	a5, 0x90(sp)
   118cc:      	lw	a6, 0xc0(sp)
   118d0:      	xor	a6, a5, a6
   118d4:      	sw	s1, 0x24(sp)
   118d8:      	sw	s9, 0x28(sp)
   118dc:      	xor	t2, s9, s1
   118e0:      	xor	a6, t2, a6
   118e4:      	lw	a5, 0xac(sp)
   118e8:      	lw	t1, 0xc8(sp)
   118ec:      	xor	t2, a5, t1
   118f0:      	sw	t4, 0x38(sp)
   118f4:      	lw	a5, 0x70(sp)
   118f8:      	xor	t5, t4, a5
   118fc:      	xor	t6, t5, t2
   11900:      	lw	a5, 0xc4(sp)
   11904:      	lw	t1, 0x94(sp)
   11908:      	xor	t2, t1, a5
   1190c:      	lw	a5, 0x9c(sp)
   11910:      	lw	t1, 0x74(sp)
   11914:      	xor	t5, a5, t1
   11918:      	xor	s3, t5, t2
   1191c:      	lw	a1, 0xd8(sp)
   11920:      	lw	a5, 0xcc(sp)
   11924:      	xor	t2, a1, a5
   11928:      	lw	a1, 0x7c(sp)
   1192c:      	lw	a5, 0x8c(sp)
   11930:      	xor	t5, a1, a5
   11934:      	xor	s6, t5, t2
   11938:      	lw	a1, 0xd4(sp)
   1193c:      	lw	t1, 0x98(sp)
   11940:      	xor	t2, t1, a1
   11944:      	lw	a1, 0xa8(sp)
   11948:      	lw	a5, 0xa4(sp)
   1194c:      	xor	t5, a5, a1
   11950:      	xor	s9, t5, t2
   11954:      	lw	s4, 0x6c(sp)
   11958:      	lw	a1, 0xbc(sp)
   1195c:      	xor	t2, a1, s4
   11960:      	lw	a1, 0x84(sp)
   11964:      	lw	a5, 0xd0(sp)
   11968:      	xor	t5, a1, a5
   1196c:      	mv	a1, a3
   11970:      	sw	a3, 0x18(sp)
   11974:      	xor	a3, t5, t2
   11978:      	sw	s10, 0x10(sp)
   1197c:      	xor	t2, s10, a4
   11980:      	sw	s0, 0x14(sp)
   11984:      	xor	t5, s0, a2
   11988:      	srli	a2, t5, 0x1f
   1198c:      	slli	a4, t2, 0x1
   11990:      	or	t3, a4, a2
   11994:      	slli	a2, t5, 0x1
   11998:      	srli	a4, t2, 0x1f
   1199c:      	or	s5, a2, a4
   119a0:      	lw	a2, 0xa0(sp)
   119a4:      	xor	a2, a2, t6
   119a8:      	lw	t1, 0x5c(sp)
   119ac:      	xor	t6, t1, a6
   119b0:      	slli	a4, t6, 0x1
   119b4:      	srli	a6, a2, 0x1f
   119b8:      	or	s10, a4, a6
   119bc:      	srli	a4, t6, 0x1f
   119c0:      	slli	a6, a2, 0x1
   119c4:      	mv	t4, a0
   119c8:      	or	ra, a6, a4
   119cc:      	lw	a0, 0x80(sp)
   119d0:      	xor	s6, a0, s6
   119d4:      	lw	s7, 0x60(sp)
   119d8:      	xor	s8, s7, s3
   119dc:      	srli	a4, s8, 0x1f
   119e0:      	slli	a6, s6, 0x1
   119e4:      	or	a5, a6, a4
   119e8:      	slli	a6, s8, 0x1
   119ec:      	srli	s3, s6, 0x1f
   119f0:      	or	a6, a6, s3
   119f4:      	xor	a0, a1, a3
   119f8:      	lw	a1, 0x88(sp)
   119fc:      	xor	s3, a1, s9
   11a00:      	slli	s9, s3, 0x1
   11a04:      	srli	a3, a0, 0x1f
   11a08:      	or	a1, s9, a3
   11a0c:      	srli	s9, s3, 0x1f
   11a10:      	slli	s0, a0, 0x1
   11a14:      	or	s0, s0, s9
   11a18:      	lw	a4, 0x54(sp)
   11a1c:      	xor	t0, a4, t0
   11a20:      	mv	s1, s2
   11a24:      	lw	a3, 0x58(sp)
   11a28:      	xor	s11, a3, a7
   11a2c:      	slli	s9, t0, 0x1
   11a30:      	srli	s2, s11, 0x1f
   11a34:      	or	s2, s9, s2
   11a38:      	srli	s9, t0, 0x1f
   11a3c:      	slli	a7, s11, 0x1
   11a40:      	or	a7, a7, s9
   11a44:      	xor	s3, s3, s5
   11a48:      	xor	a0, a0, t3
   11a4c:      	xor	s5, ra, s11
   11a50:      	xor	t0, s10, t0
   11a54:      	xor	s11, a6, t5
   11a58:      	xor	t2, a5, t2
   11a5c:      	xor	s0, s0, a2
   11a60:      	xor	a2, a1, t6
   11a64:      	xor	a7, s6, a7
   11a68:      	xor	a1, s8, s2
   11a6c:      	lw	a5, 0x44(sp)
   11a70:      	xor	a5, a0, a5
   11a74:      	sw	a5, 0x44(sp)
   11a78:      	lw	a5, 0xb8(sp)
   11a7c:      	xor	ra, a0, a5
   11a80:      	lw	a5, 0x34(sp)
   11a84:      	xor	t6, a0, a5
   11a88:      	lw	a5, 0x30(sp)
   11a8c:      	xor	a5, a0, a5
   11a90:      	sw	a5, 0x34(sp)
   11a94:      	xor	a0, a0, a3
   11a98:      	sw	a0, 0x30(sp)
   11a9c:      	lw	a0, 0x1c(sp)
   11aa0:      	xor	a0, s3, a0
   11aa4:      	sw	a0, 0xb8(sp)
   11aa8:      	lw	a0, 0xb4(sp)
   11aac:      	xor	s10, s3, a0
   11ab0:      	lw	a0, 0x3c(sp)
   11ab4:      	xor	t3, s3, a0
   11ab8:      	lw	a0, 0x2c(sp)
   11abc:      	xor	a0, s3, a0
   11ac0:      	sw	a0, 0x1c(sp)
   11ac4:      	xor	a0, s3, a4
   11ac8:      	sw	a0, 0x2c(sp)
   11acc:      	xor	a5, s5, t4
   11ad0:      	lw	a0, 0xb0(sp)
   11ad4:      	xor	a0, s5, a0
   11ad8:      	sw	a0, 0xb0(sp)
   11adc:      	lw	a0, 0x78(sp)
   11ae0:      	xor	s2, s5, a0
   11ae4:      	lw	a0, 0x20(sp)
   11ae8:      	xor	s8, s5, a0
   11aec:      	lw	a0, 0x10(sp)
   11af0:      	xor	a0, s5, a0
   11af4:      	sw	a0, 0x58(sp)
   11af8:      	xor	a3, t0, s1
   11afc:      	lw	a0, 0x40(sp)
   11b00:      	xor	a0, t0, a0
   11b04:      	sw	a0, 0x68(sp)
   11b08:      	lw	a0, 0x48(sp)
   11b0c:      	xor	t5, t0, a0
   11b10:      	lw	a0, 0x4c(sp)
   11b14:      	xor	s6, t0, a0
   11b18:      	lw	a0, 0x14(sp)
   11b1c:      	xor	a0, t0, a0
   11b20:      	sw	a0, 0x54(sp)
   11b24:      	lw	a0, 0xc0(sp)
   11b28:      	xor	a0, s11, a0
   11b2c:      	sw	a0, 0x64(sp)
   11b30:      	lw	a0, 0x90(sp)
   11b34:      	xor	a6, s11, a0
   11b38:      	lw	a0, 0x24(sp)
   11b3c:      	xor	a0, s11, a0
   11b40:      	sw	a0, 0x3c(sp)
   11b44:      	lw	a0, 0x28(sp)
   11b48:      	xor	t4, s11, a0
   11b4c:      	xor	a0, s11, t1
   11b50:      	sw	a0, 0xc0(sp)
   11b54:      	lw	a0, 0xc8(sp)
   11b58:      	xor	a0, t2, a0
   11b5c:      	sw	a0, 0x90(sp)
   11b60:      	lw	a0, 0xac(sp)
   11b64:      	xor	a4, t2, a0
   11b68:      	lw	a0, 0x70(sp)
   11b6c:      	xor	a0, t2, a0
   11b70:      	sw	a0, 0x70(sp)
   11b74:      	lw	a0, 0x38(sp)
   11b78:      	xor	t0, t2, a0
   11b7c:      	lw	a0, 0xa0(sp)
   11b80:      	xor	a0, t2, a0
   11b84:      	sw	a0, 0xc8(sp)
   11b88:      	lw	s1, 0xc4(sp)
   11b8c:      	xor	s1, a2, s1
   11b90:      	lw	a0, 0x94(sp)
   11b94:      	xor	s5, a2, a0
   11b98:      	lw	a0, 0x74(sp)
   11b9c:      	xor	a0, a2, a0
   11ba0:      	sw	a0, 0x28(sp)
   11ba4:      	lw	a0, 0x9c(sp)
   11ba8:      	xor	t1, a2, a0
   11bac:      	xor	a0, a2, s7
   11bb0:      	sw	a0, 0x38(sp)
   11bb4:      	lw	a0, 0xcc(sp)
   11bb8:      	xor	t2, s0, a0
   11bbc:      	lw	a0, 0xd8(sp)
   11bc0:      	xor	s3, s0, a0
   11bc4:      	lw	a0, 0x8c(sp)
   11bc8:      	xor	a0, s0, a0
   11bcc:      	sw	a0, 0x24(sp)
   11bd0:      	lw	a2, 0x7c(sp)
   11bd4:      	xor	a2, s0, a2
   11bd8:      	lw	a0, 0x80(sp)
   11bdc:      	xor	a0, s0, a0
   11be0:      	sw	a0, 0x20(sp)
   11be4:      	xor	s7, s4, a7
   11be8:      	lw	a0, 0xbc(sp)
   11bec:      	xor	a0, a0, a7
   11bf0:      	sw	a0, 0x94(sp)
   11bf4:      	lw	a0, 0xd0(sp)
   11bf8:      	xor	a0, a0, a7
   11bfc:      	sw	a0, 0x74(sp)
   11c00:      	lw	a0, 0x84(sp)
   11c04:      	xor	s11, a0, a7
   11c08:      	lw	a0, 0x18(sp)
   11c0c:      	xor	s0, a0, a7
   11c10:      	lw	a0, 0xd4(sp)
   11c14:      	xor	s4, a0, a1
   11c18:      	lw	a0, 0x98(sp)
   11c1c:      	xor	a0, a0, a1
   11c20:      	sw	a0, 0xd4(sp)
   11c24:      	lw	a0, 0xa8(sp)
   11c28:      	xor	a0, a0, a1
   11c2c:      	sw	a0, 0x98(sp)
   11c30:      	lw	a0, 0xa4(sp)
   11c34:      	xor	s9, a0, a1
   11c38:      	lw	a0, 0x88(sp)
   11c3c:      	xor	a7, a0, a1
   11c40:      	srli	a1, a5, 0x1f
   11c44:      	slli	a0, a3, 0x1
   11c48:      	or	a0, a0, a1
   11c4c:      	sw	a0, 0xd0(sp)
   11c50:      	srli	a3, a3, 0x1f
   11c54:      	slli	a5, a5, 0x1
   11c58:      	or	a3, a5, a3
   11c5c:      	sw	a3, 0xa8(sp)
   11c60:      	srli	a0, t3, 0x1d
   11c64:      	slli	a1, t6, 0x3
   11c68:      	or	a0, a1, a0
   11c6c:      	sw	a0, 0xbc(sp)
   11c70:      	srli	a0, t6, 0x1d
   11c74:      	slli	t3, t3, 0x3
   11c78:      	or	a0, t3, a0
   11c7c:      	sw	a0, 0xd8(sp)
   11c80:      	srli	a0, a4, 0x1a
   11c84:      	slli	a1, a6, 0x6
   11c88:      	or	a0, a1, a0
   11c8c:      	sw	a0, 0x8c(sp)
   11c90:      	srli	a0, a6, 0x1a
   11c94:      	slli	a4, a4, 0x6
   11c98:      	or	a0, a4, a0
   11c9c:      	sw	a0, 0x78(sp)
   11ca0:      	srli	a0, t5, 0x16
   11ca4:      	slli	a1, s2, 0xa
   11ca8:      	or	a0, a1, a0
   11cac:      	sw	a0, 0x84(sp)
   11cb0:      	srli	a0, s2, 0x16
   11cb4:      	slli	t5, t5, 0xa
   11cb8:      	or	a0, t5, a0
   11cbc:      	sw	a0, 0xa0(sp)
   11cc0:      	srli	a0, t4, 0x11
   11cc4:      	slli	a1, t0, 0xf
   11cc8:      	or	a0, a1, a0
   11ccc:      	sw	a0, 0x9c(sp)
   11cd0:      	srli	a0, t0, 0x11
   11cd4:      	slli	t4, t4, 0xf
   11cd8:      	or	a0, t4, a0
   11cdc:      	sw	a0, 0xa4(sp)
   11ce0:      	srli	a0, t1, 0xb
   11ce4:      	slli	a1, a2, 0x15
   11ce8:      	or	a0, a1, a0
   11cec:      	sw	a0, 0xcc(sp)
   11cf0:      	srli	a2, a2, 0xb
   11cf4:      	slli	t1, t1, 0x15
   11cf8:      	or	a0, t1, a2
   11cfc:      	sw	a0, 0xc4(sp)
   11d00:      	srli	a0, t2, 0x4
   11d04:      	slli	a1, s1, 0x1c
   11d08:      	or	a0, a1, a0
   11d0c:      	sw	a0, 0xac(sp)
   11d10:      	srli	s1, s1, 0x4
   11d14:      	slli	t2, t2, 0x1c
   11d18:      	or	a0, t2, s1
   11d1c:      	sw	a0, 0xb4(sp)
   11d20:      	srli	a0, s10, 0x1c
   11d24:      	slli	a1, ra, 0x4
   11d28:      	or	a0, a1, a0
   11d2c:      	sw	a0, 0x7c(sp)
   11d30:      	srli	a0, ra, 0x1c
   11d34:      	slli	s10, s10, 0x4
   11d38:      	or	a0, s10, a0
   11d3c:      	sw	a0, 0x5c(sp)
   11d40:      	srli	a0, s8, 0x13
   11d44:      	slli	a1, s6, 0xd
   11d48:      	or	s10, a1, a0
   11d4c:      	srli	a0, s6, 0x13
   11d50:      	slli	s8, s8, 0xd
   11d54:      	or	ra, s8, a0
   11d58:      	lw	s8, 0x50(sp)
   11d5c:      	srli	a0, s5, 0x9
   11d60:      	slli	a1, s3, 0x17
   11d64:      	or	a0, a1, a0
   11d68:      	sw	a0, 0x88(sp)
   11d6c:      	srli	a0, s3, 0x9
   11d70:      	slli	s5, s5, 0x17
   11d74:      	or	a0, s5, a0
   11d78:      	sw	a0, 0x80(sp)
   11d7c:      	lw	a3, 0x54(sp)
   11d80:      	srli	a0, a3, 0x1e
   11d84:      	lw	a2, 0x58(sp)
   11d88:      	slli	a1, a2, 0x2
   11d8c:      	or	a0, a1, a0
   11d90:      	sw	a0, 0x48(sp)
   11d94:      	srli	a0, a2, 0x1e
   11d98:      	slli	a1, a3, 0x2
   11d9c:      	or	a0, a1, a0
   11da0:      	sw	a0, 0x4c(sp)
   11da4:      	srli	a0, a7, 0x12
   11da8:      	slli	a1, s0, 0xe
   11dac:      	or	t2, a1, a0
   11db0:      	srli	s0, s0, 0x12
   11db4:      	slli	a7, a7, 0xe
   11db8:      	or	a7, a7, s0
   11dbc:      	srli	a0, s7, 0x5
   11dc0:      	slli	a1, s4, 0x1b
   11dc4:      	or	a0, a1, a0
   11dc8:      	sw	a0, 0x58(sp)
   11dcc:      	srli	a0, s4, 0x5
   11dd0:      	slli	s7, s7, 0x1b
   11dd4:      	or	a0, s7, a0
   11dd8:      	sw	a0, 0x54(sp)
   11ddc:      	lw	a2, 0x34(sp)
   11de0:      	srli	a0, a2, 0x17
   11de4:      	lw	a3, 0x1c(sp)
   11de8:      	slli	a1, a3, 0x9
   11dec:      	or	a0, a1, a0
   11df0:      	sw	a0, 0x40(sp)
   11df4:      	srli	a0, a3, 0x17
   11df8:      	slli	a1, a2, 0x9
   11dfc:      	or	a0, a1, a0
   11e00:      	sw	a0, 0x60(sp)
   11e04:      	lw	a3, 0x20(sp)
   11e08:      	srli	a0, a3, 0x8
   11e0c:      	lw	a2, 0x38(sp)
   11e10:      	slli	a1, a2, 0x18
   11e14:      	or	a0, a1, a0
   11e18:      	sw	a0, 0x34(sp)
   11e1c:      	srli	a0, a2, 0x8
   11e20:      	slli	a1, a3, 0x18
   11e24:      	or	a0, a1, a0
   11e28:      	sw	a0, 0x38(sp)
   11e2c:      	srli	a0, s9, 0x18
   11e30:      	slli	a1, s11, 0x8
   11e34:      	or	t5, a1, a0
   11e38:      	srli	a0, s11, 0x18
   11e3c:      	slli	s9, s9, 0x8
   11e40:      	or	s4, s9, a0
   11e44:      	lw	s1, 0x28(sp)
   11e48:      	srli	a0, s1, 0x7
   11e4c:      	lw	a2, 0x24(sp)
   11e50:      	slli	a1, a2, 0x19
   11e54:      	or	s2, a1, a0
   11e58:      	srli	a0, a2, 0x7
   11e5c:      	slli	s1, s1, 0x19
   11e60:      	or	s1, s1, a0
   11e64:      	lw	a1, 0x3c(sp)
   11e68:      	srli	a0, a1, 0x15
   11e6c:      	lw	a2, 0x70(sp)
   11e70:      	slli	a4, a2, 0xb
   11e74:      	or	a4, a4, a0
   11e78:      	srli	a0, a2, 0x15
   11e7c:      	slli	a1, a1, 0xb
   11e80:      	or	t1, a1, a0
   11e84:      	lw	a2, 0x64(sp)
   11e88:      	srli	a0, a2, 0x2
   11e8c:      	lw	a3, 0x90(sp)
   11e90:      	slli	a1, a3, 0x1e
   11e94:      	or	a0, a1, a0
   11e98:      	sw	a0, 0x3c(sp)
   11e9c:      	srli	a0, a3, 0x2
   11ea0:      	slli	a1, a2, 0x1e
   11ea4:      	or	t3, a1, a0
   11ea8:      	lw	a2, 0x2c(sp)
   11eac:      	srli	a0, a2, 0xe
   11eb0:      	lw	a1, 0x30(sp)
   11eb4:      	slli	a5, a1, 0x12
   11eb8:      	or	a5, a5, a0
   11ebc:      	srli	a0, a1, 0xe
   11ec0:      	slli	a1, a2, 0x12
   11ec4:      	or	t4, a1, a0
   11ec8:      	lw	a1, 0x74(sp)
   11ecc:      	srli	a0, a1, 0x19
   11ed0:      	lw	a2, 0x98(sp)
   11ed4:      	slli	s0, a2, 0x7
   11ed8:      	or	a0, s0, a0
   11edc:      	sw	a0, 0x2c(sp)
   11ee0:      	srli	a0, a2, 0x19
   11ee4:      	slli	a1, a1, 0x7
   11ee8:      	or	a0, a1, a0
   11eec:      	sw	a0, 0x30(sp)
   11ef0:      	lw	a3, 0xc8(sp)
   11ef4:      	srli	a0, a3, 0x3
   11ef8:      	lw	a1, 0xc0(sp)
   11efc:      	slli	a2, a1, 0x1d
   11f00:      	or	a2, a2, a0
   11f04:      	srli	a0, a1, 0x3
   11f08:      	slli	a1, a3, 0x1d
   11f0c:      	or	a6, a1, a0
   11f10:      	lw	a1, 0x94(sp)
   11f14:      	srli	a0, a1, 0xc
   11f18:      	lw	t0, 0xd4(sp)
   11f1c:      	slli	a3, t0, 0x14
   11f20:      	or	a3, a3, a0
   11f24:      	srli	a0, t0, 0xc
   11f28:      	slli	a1, a1, 0x14
   11f2c:      	or	a1, a1, a0
   11f30:      	lw	t6, 0xb0(sp)
   11f34:      	srli	a0, t6, 0x14
   11f38:      	lw	s0, 0x68(sp)
   11f3c:      	slli	t0, s0, 0xc
   11f40:      	or	a0, t0, a0
   11f44:      	lw	s3, 0x4(s8)
   11f48:      	srli	t0, s0, 0x14
   11f4c:      	slli	t6, t6, 0xc
   11f50:      	or	t0, t6, t0
   11f54:      	lw	t6, 0x44(sp)
   11f58:      	xor	s0, s3, t6
   11f5c:      	sw	s0, 0x28(sp)
   11f60:      	not	s3, t2
   11f64:      	and	s3, t6, s3
   11f68:      	not	s5, t6
   11f6c:      	and	s5, a0, s5
   11f70:      	lw	s6, 0x0(s8)
   11f74:      	lw	s0, 0xcc(sp)
   11f78:      	not	s7, s0
   11f7c:      	and	s7, t2, s7
   11f80:      	xor	t2, s5, t2
   11f84:      	sw	t2, 0x6c(sp)
   11f88:      	lw	t6, 0xb8(sp)
   11f8c:      	xor	t2, s6, t6
   11f90:      	sw	t2, 0x44(sp)
   11f94:      	not	t2, a7
   11f98:      	and	s5, t6, t2
   11f9c:      	not	t2, t6
   11fa0:      	and	t2, t0, t2
   11fa4:      	lw	t6, 0xc4(sp)
   11fa8:      	not	s6, t6
   11fac:      	and	s6, a7, s6
   11fb0:      	xor	a7, t2, a7
   11fb4:      	sw	a7, 0xd4(sp)
   11fb8:      	not	a7, t1
   11fbc:      	and	a7, s0, a7
   11fc0:      	xor	a7, a7, a0
   11fc4:      	sw	a7, 0x64(sp)
   11fc8:      	not	a0, a0
   11fcc:      	and	t2, t1, a0
   11fd0:      	xor	a0, s7, t1
   11fd4:      	sw	a0, 0xc8(sp)
   11fd8:      	not	a0, a4
   11fdc:      	and	a0, t6, a0
   11fe0:      	xor	a0, a0, t0
   11fe4:      	sw	a0, 0x68(sp)
   11fe8:      	not	a0, t0
   11fec:      	and	t0, a4, a0
   11ff0:      	xor	a0, s6, a4
   11ff4:      	sw	a0, 0xc0(sp)
   11ff8:      	xor	a0, s0, s3
   11ffc:      	sw	a0, 0xcc(sp)
   12000:      	xor	a0, t6, s5
   12004:      	sw	a0, 0xc4(sp)
   12008:      	not	a0, a1
   1200c:      	lw	t6, 0xbc(sp)
   12010:      	and	a0, t6, a0
   12014:      	not	a4, a2
   12018:      	lw	a7, 0xb4(sp)
   1201c:      	and	a4, a7, a4
   12020:      	xor	a0, a0, a7
   12024:      	sw	a0, 0xb8(sp)
   12028:      	not	a0, a7
   1202c:      	not	a7, t6
   12030:      	and	a7, s10, a7
   12034:      	and	a0, a1, a0
   12038:      	xor	a1, a7, a1
   1203c:      	sw	a1, 0xb0(sp)
   12040:      	not	a1, a3
   12044:      	lw	s0, 0xd8(sp)
   12048:      	and	a1, s0, a1
   1204c:      	not	a7, a6
   12050:      	lw	t1, 0xac(sp)
   12054:      	and	a7, t1, a7
   12058:      	xor	a1, a1, t1
   1205c:      	sw	a1, 0xb4(sp)
   12060:      	not	a1, t1
   12064:      	not	t1, s0
   12068:      	and	t1, ra, t1
   1206c:      	and	a1, a3, a1
   12070:      	xor	t1, t1, a3
   12074:      	not	a3, s10
   12078:      	and	a3, a2, a3
   1207c:      	xor	a3, t6, a3
   12080:      	sw	a3, 0xac(sp)
   12084:      	not	a3, ra
   12088:      	and	a3, a6, a3
   1208c:      	xor	a3, s0, a3
   12090:      	sw	a3, 0x90(sp)
   12094:      	xor	a3, a4, s10
   12098:      	sw	a3, 0xd8(sp)
   1209c:      	xor	a3, a7, ra
   120a0:      	sw	a3, 0x94(sp)
   120a4:      	xor	a0, a0, a2
   120a8:      	sw	a0, 0xbc(sp)
   120ac:      	xor	a0, a1, a6
   120b0:      	sw	a0, 0x98(sp)
   120b4:      	lw	a3, 0x78(sp)
   120b8:      	not	a0, a3
   120bc:      	and	a0, s2, a0
   120c0:      	not	a1, a5
   120c4:      	lw	a2, 0xa8(sp)
   120c8:      	and	a1, a2, a1
   120cc:      	xor	a7, a0, a2
   120d0:      	not	a0, a2
   120d4:      	not	a2, s2
   120d8:      	and	a2, t5, a2
   120dc:      	and	a0, a3, a0
   120e0:      	xor	a2, a2, a3
   120e4:      	sw	a2, 0x78(sp)
   120e8:      	lw	a6, 0x8c(sp)
   120ec:      	not	a2, a6
   120f0:      	and	a2, s1, a2
   120f4:      	not	a3, t4
   120f8:      	lw	a4, 0xd0(sp)
   120fc:      	and	a3, a4, a3
   12100:      	xor	ra, a2, a4
   12104:      	not	a2, a4
   12108:      	not	a4, s1
   1210c:      	and	a4, s4, a4
   12110:      	and	a2, a6, a2
   12114:      	xor	s7, a4, a6
   12118:      	not	a4, t5
   1211c:      	and	a4, a5, a4
   12120:      	xor	a4, a4, s2
   12124:      	sw	a4, 0x70(sp)
   12128:      	not	a4, s4
   1212c:      	and	a4, t4, a4
   12130:      	xor	s1, a4, s1
   12134:      	xor	a1, a1, t5
   12138:      	sw	a1, 0x8c(sp)
   1213c:      	xor	a1, a3, s4
   12140:      	sw	a1, 0x74(sp)
   12144:      	xor	a0, a5, a0
   12148:      	sw	a0, 0xd0(sp)
   1214c:      	xor	a0, t4, a2
   12150:      	sw	a0, 0xa8(sp)
   12154:      	lw	a4, 0x5c(sp)
   12158:      	not	a0, a4
   1215c:      	lw	a6, 0x84(sp)
   12160:      	and	a0, a6, a0
   12164:      	lw	t6, 0x34(sp)
   12168:      	not	a1, t6
   1216c:      	lw	a2, 0x54(sp)
   12170:      	and	a1, a2, a1
   12174:      	xor	s6, a0, a2
   12178:      	not	a0, a2
   1217c:      	not	a2, a6
   12180:      	lw	s4, 0x9c(sp)
   12184:      	and	a2, s4, a2
   12188:      	and	a0, a4, a0
   1218c:      	xor	s5, a4, a2
   12190:      	lw	s0, 0x7c(sp)
   12194:      	not	a2, s0
   12198:      	lw	s2, 0xa0(sp)
   1219c:      	and	a2, s2, a2
   121a0:      	lw	s10, 0x38(sp)
   121a4:      	not	a3, s10
   121a8:      	lw	a4, 0x58(sp)
   121ac:      	and	a3, a4, a3
   121b0:      	xor	t5, a2, a4
   121b4:      	not	a2, a4
   121b8:      	not	a4, s2
   121bc:      	lw	s9, 0xa4(sp)
   121c0:      	and	a4, s9, a4
   121c4:      	and	a2, s0, a2
   121c8:      	xor	s11, s0, a4
   121cc:      	not	a4, s4
   121d0:      	and	a4, t6, a4
   121d4:      	xor	t4, a4, a6
   121d8:      	not	a4, s9
   121dc:      	mv	a6, s9
   121e0:      	and	a4, s10, a4
   121e4:      	xor	s9, a4, s2
   121e8:      	xor	a1, a1, s4
   121ec:      	sw	a1, 0x7c(sp)
   121f0:      	xor	a1, a3, a6
   121f4:      	sw	a1, 0x9c(sp)
   121f8:      	xor	a0, a0, t6
   121fc:      	sw	a0, 0x84(sp)
   12200:      	xor	a0, a2, s10
   12204:      	sw	a0, 0xa4(sp)
   12208:      	lw	a3, 0x80(sp)
   1220c:      	not	a0, a3
   12210:      	lw	t6, 0x2c(sp)
   12214:      	and	a0, t6, a0
   12218:      	lw	s4, 0x48(sp)
   1221c:      	not	a1, s4
   12220:      	and	a1, t3, a1
   12224:      	xor	a0, a0, t3
   12228:      	sw	a0, 0x58(sp)
   1222c:      	not	a0, t3
   12230:      	not	a2, t6
   12234:      	lw	t3, 0x40(sp)
   12238:      	and	a2, t3, a2
   1223c:      	and	a0, a3, a0
   12240:      	xor	s10, a2, a3
   12244:      	lw	s0, 0x88(sp)
   12248:      	not	a2, s0
   1224c:      	lw	s3, 0x30(sp)
   12250:      	and	a2, s3, a2
   12254:      	lw	a6, 0x4c(sp)
   12258:      	not	a3, a6
   1225c:      	lw	a4, 0x3c(sp)
   12260:      	and	a3, a4, a3
   12264:      	xor	a2, a2, a4
   12268:      	sw	a2, 0x54(sp)
   1226c:      	not	a2, a4
   12270:      	not	a4, s3
   12274:      	lw	s2, 0x60(sp)
   12278:      	and	a4, s2, a4
   1227c:      	and	a2, s0, a2
   12280:      	xor	s0, a4, s0
   12284:      	not	a4, t3
   12288:      	and	a4, s4, a4
   1228c:      	xor	a4, a4, t6
   12290:      	sw	a4, 0xa0(sp)
   12294:      	not	a4, s2
   12298:      	and	a4, a6, a4
   1229c:      	xor	a4, a4, s3
   122a0:      	sw	a4, 0x5c(sp)
   122a4:      	xor	a1, t3, a1
   122a8:      	sw	a1, 0x80(sp)
   122ac:      	xor	a1, s2, a3
   122b0:      	sw	a1, 0x60(sp)
   122b4:      	xor	a3, a0, s4
   122b8:      	xor	a0, a2, a6
   122bc:      	sw	a0, 0x88(sp)
   122c0:      	lw	a0, 0x28(sp)
   122c4:      	xor	a5, t2, a0
   122c8:      	addi	s8, s8, 0x8
   122cc:      	lw	a0, 0x44(sp)
   122d0:      	xor	a6, t0, a0
   122d4:      	lw	a0, 0xc(sp)
   122d8:      	bne	s8, a0, 0x11854 <keccak::p1600::he90e6ecc89f09623+0x1b8>
   122dc:      	lw	a0, 0x8(sp)
   122e0:      	sw	ra, 0x50(a0)
   122e4:      	sw	a7, 0x54(a0)
   122e8:      	sw	s7, 0x58(a0)
   122ec:      	lw	a1, 0x78(sp)
   122f0:      	sw	a1, 0x5c(a0)
   122f4:      	lw	a1, 0x54(sp)
   122f8:      	sw	a1, 0xa0(a0)
   122fc:      	lw	a1, 0x58(sp)
   12300:      	sw	a1, 0xa4(a0)
   12304:      	sw	s0, 0xa8(a0)
   12308:      	sw	s10, 0xac(a0)
   1230c:      	sw	t1, 0x30(a0)
   12310:      	lw	a1, 0xb0(sp)
   12314:      	sw	a1, 0x34(a0)
   12318:      	lw	a1, 0x90(sp)
   1231c:      	sw	a1, 0x38(a0)
   12320:      	lw	a1, 0xac(sp)
   12324:      	sw	a1, 0x3c(a0)
   12328:      	sw	s11, 0x80(a0)
   1232c:      	sw	s5, 0x84(a0)
   12330:      	sw	s9, 0x88(a0)
   12334:      	sw	t4, 0x8c(a0)
   12338:      	lw	a1, 0xc0(sp)
   1233c:      	sw	a1, 0x10(a0)
   12340:      	lw	a1, 0xc8(sp)
   12344:      	sw	a1, 0x14(a0)
   12348:      	lw	a1, 0xc4(sp)
   1234c:      	sw	a1, 0x18(a0)
   12350:      	lw	a1, 0xcc(sp)
   12354:      	sw	a1, 0x1c(a0)
   12358:      	sw	s1, 0x60(a0)
   1235c:      	lw	a1, 0x70(sp)
   12360:      	sw	a1, 0x64(a0)
   12364:      	lw	a1, 0x74(sp)
   12368:      	sw	a1, 0x68(a0)
   1236c:      	lw	a1, 0x8c(sp)
   12370:      	sw	a1, 0x6c(a0)
   12374:      	lw	a1, 0xd4(sp)
   12378:      	sw	a1, 0x20(a0)
   1237c:      	lw	a1, 0x6c(sp)
   12380:      	sw	a1, 0x24(a0)
   12384:      	lw	a1, 0xb4(sp)
   12388:      	sw	a1, 0x28(a0)
   1238c:      	lw	a1, 0xb8(sp)
   12390:      	sw	a1, 0x2c(a0)
   12394:      	lw	a1, 0xa8(sp)
   12398:      	sw	a1, 0x70(a0)
   1239c:      	lw	a1, 0xd0(sp)
   123a0:      	sw	a1, 0x74(a0)
   123a4:      	sw	t5, 0x78(a0)
   123a8:      	sw	s6, 0x7c(a0)
   123ac:      	sw	a6, 0x0(a0)
   123b0:      	sw	a5, 0x4(a0)
   123b4:      	lw	a1, 0x68(sp)
   123b8:      	sw	a1, 0x8(a0)
   123bc:      	lw	a1, 0x64(sp)
   123c0:      	sw	a1, 0xc(a0)
   123c4:      	lw	a1, 0x5c(sp)
   123c8:      	sw	a1, 0xb0(a0)
   123cc:      	lw	a1, 0xa0(sp)
   123d0:      	sw	a1, 0xb4(a0)
   123d4:      	lw	a1, 0x60(sp)
   123d8:      	sw	a1, 0xb8(a0)
   123dc:      	lw	a1, 0x80(sp)
   123e0:      	sw	a1, 0xbc(a0)
   123e4:      	lw	a1, 0x94(sp)
   123e8:      	sw	a1, 0x40(a0)
   123ec:      	lw	a1, 0xd8(sp)
   123f0:      	sw	a1, 0x44(a0)
   123f4:      	lw	a1, 0x98(sp)
   123f8:      	sw	a1, 0x48(a0)
   123fc:      	lw	a1, 0xbc(sp)
   12400:      	sw	a1, 0x4c(a0)
   12404:      	lw	a1, 0x9c(sp)
   12408:      	sw	a1, 0x90(a0)
   1240c:      	lw	a1, 0x7c(sp)
   12410:      	sw	a1, 0x94(a0)
   12414:      	lw	a1, 0xa4(sp)
   12418:      	sw	a1, 0x98(a0)
   1241c:      	lw	a1, 0x84(sp)
   12420:      	sw	a1, 0x9c(a0)
   12424:      	lw	a1, 0x88(sp)
   12428:      	sw	a1, 0xc0(a0)
   1242c:      	sw	a3, 0xc4(a0)
   12430:      	lw	ra, 0x10c(sp)
   12434:      	lw	s0, 0x108(sp)
   12438:      	lw	s1, 0x104(sp)
   1243c:      	lw	s2, 0x100(sp)
   12440:      	lw	s3, 0xfc(sp)
   12444:      	lw	s4, 0xf8(sp)
   12448:      	lw	s5, 0xf4(sp)
   1244c:      	lw	s6, 0xf0(sp)
   12450:      	lw	s7, 0xec(sp)
   12454:      	lw	s8, 0xe8(sp)
   12458:      	lw	s9, 0xe4(sp)
   1245c:      	lw	s10, 0xe0(sp)
   12460:      	lw	s11, 0xdc(sp)
   12464:      	addi	sp, sp, 0x110
   12468:      	ret
   1246c:      	auipc	ra, 0xfffff
   12470:      	jalr	0x218(ra) <memset+0xffffefcc>

00012474 <memcpy>:
   12474:      	addi	sp, sp, -0x10
   12478:      	sw	ra, 0xc(sp)
   1247c:      	sw	s0, 0x8(sp)
   12480:      	addi	s0, sp, 0x10
   12484:      	lw	ra, 0xc(sp)
   12488:      	lw	s0, 0x8(sp)
   1248c:      	addi	sp, sp, 0x10
   12490:      	auipc	t1, 0x0
   12494:      	jr	0x8(t1) <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c>

00012498 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c>:
   12498:      	addi	sp, sp, -0x20
   1249c:      	sw	ra, 0x1c(sp)
   124a0:      	sw	s0, 0x18(sp)
   124a4:      	addi	s0, sp, 0x20
   124a8:      	li	a3, 0x10
   124ac:      	bltu	a2, a3, 0x1252c <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x94>
   124b0:      	neg	a3, a0
   124b4:      	andi	a3, a3, 0x3
   124b8:      	add	a5, a0, a3
   124bc:      	bgeu	a0, a5, 0x124e4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x4c>
   124c0:      	mv	a4, a3
   124c4:      	mv	a6, a0
   124c8:      	mv	a7, a1
   124cc:      	lbu	t0, 0x0(a7)
   124d0:      	addi	a4, a4, -0x1
   124d4:      	sb	t0, 0x0(a6)
   124d8:      	addi	a6, a6, 0x1
   124dc:      	addi	a7, a7, 0x1
   124e0:      	bnez	a4, 0x124cc <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x34>
   124e4:      	add	a1, a1, a3
   124e8:      	sub	a2, a2, a3
   124ec:      	andi	a4, a2, -0x4
   124f0:      	andi	a7, a1, 0x3
   124f4:      	add	a3, a5, a4
   124f8:      	bnez	a7, 0x12560 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xc8>
   124fc:      	bgeu	a5, a3, 0x12518 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x80>
   12500:      	mv	a6, a1
   12504:      	lw	a7, 0x0(a6)
   12508:      	sw	a7, 0x0(a5)
   1250c:      	addi	a5, a5, 0x4
   12510:      	addi	a6, a6, 0x4
   12514:      	bltu	a5, a3, 0x12504 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x6c>
   12518:      	add	a1, a1, a4
   1251c:      	andi	a2, a2, 0x3
   12520:      	add	a4, a3, a2
   12524:      	bltu	a3, a4, 0x12538 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   12528:      	j	0x12550 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>
   1252c:      	mv	a3, a0
   12530:      	add	a4, a0, a2
   12534:      	bgeu	a0, a4, 0x12550 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>
   12538:      	lbu	a4, 0x0(a1)
   1253c:      	addi	a2, a2, -0x1
   12540:      	sb	a4, 0x0(a3)
   12544:      	addi	a3, a3, 0x1
   12548:      	addi	a1, a1, 0x1
   1254c:      	bnez	a2, 0x12538 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   12550:      	lw	ra, 0x1c(sp)
   12554:      	lw	s0, 0x18(sp)
   12558:      	addi	sp, sp, 0x20
   1255c:      	ret
   12560:      	li	a6, 0x0
   12564:      	li	t0, 0x4
   12568:      	sw	zero, -0xc(s0)
   1256c:      	sub	t1, t0, a7
   12570:      	addi	t0, s0, -0xc
   12574:      	andi	t2, t1, 0x1
   12578:      	or	t0, t0, a7
   1257c:      	bnez	t2, 0x125d8 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x140>
   12580:      	andi	t1, t1, 0x2
   12584:      	bnez	t1, 0x125ec <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x154>
   12588:      	lw	t4, -0xc(s0)
   1258c:      	slli	a6, a7, 0x3
   12590:      	addi	t0, a5, 0x4
   12594:      	sub	t5, a1, a7
   12598:      	bgeu	t0, a3, 0x12610 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x178>
   1259c:      	neg	t0, a6
   125a0:      	andi	t3, t0, 0x18
   125a4:      	lw	t0, 0x4(t5)
   125a8:      	addi	t2, t5, 0x4
   125ac:      	srl	t4, t4, a6
   125b0:      	addi	t1, a5, 0x4
   125b4:      	sll	t5, t0, t3
   125b8:      	or	t4, t5, t4
   125bc:      	addi	t6, a5, 0x8
   125c0:      	sw	t4, 0x0(a5)
   125c4:      	mv	a5, t1
   125c8:      	mv	t5, t2
   125cc:      	mv	t4, t0
   125d0:      	bltu	t6, a3, 0x125a4 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x10c>
   125d4:      	j	0x1261c <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x184>
   125d8:      	lbu	a6, 0x0(a1)
   125dc:      	sb	a6, 0x0(t0)
   125e0:      	li	a6, 0x1
   125e4:      	andi	t1, t1, 0x2
   125e8:      	beqz	t1, 0x12588 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xf0>
   125ec:      	add	t1, a1, a6
   125f0:      	lh	t1, 0x0(t1)
   125f4:      	add	a6, t0, a6
   125f8:      	sh	t1, 0x0(a6)
   125fc:      	lw	t4, -0xc(s0)
   12600:      	slli	a6, a7, 0x3
   12604:      	addi	t0, a5, 0x4
   12608:      	sub	t5, a1, a7
   1260c:      	bltu	t0, a3, 0x1259c <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x104>
   12610:      	mv	t0, t4
   12614:      	mv	t2, t5
   12618:      	mv	t1, a5
   1261c:      	sb	zero, -0x10(s0)
   12620:      	li	a5, 0x1
   12624:      	sb	zero, -0x12(s0)
   12628:      	bne	a7, a5, 0x12640 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1a8>
   1262c:      	li	a7, 0x0
   12630:      	li	a5, 0x0
   12634:      	li	t3, 0x0
   12638:      	addi	t4, s0, -0x10
   1263c:      	j	0x12658 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1c0>
   12640:      	lbu	a7, 0x4(t2)
   12644:      	lbu	a5, 0x5(t2)
   12648:      	li	t3, 0x2
   1264c:      	sb	a7, -0x10(s0)
   12650:      	slli	a5, a5, 0x8
   12654:      	addi	t4, s0, -0x12
   12658:      	andi	t5, a1, 0x1
   1265c:      	bnez	t5, 0x12668 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1d0>
   12660:      	li	t2, 0x0
   12664:      	j	0x12684 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0x1ec>
   12668:      	addi	t2, t2, 0x4
   1266c:      	add	t2, t2, t3
   12670:      	lbu	a7, 0x0(t2)
   12674:      	sb	a7, 0x0(t4)
   12678:      	lbu	t2, -0x12(s0)
   1267c:      	lbu	a7, -0x10(s0)
   12680:      	slli	t2, t2, 0x10
   12684:      	or	a7, t2, a7
   12688:      	srl	t0, t0, a6
   1268c:      	neg	a6, a6
   12690:      	or	a5, a5, a7
   12694:      	andi	a6, a6, 0x18
   12698:      	sll	a5, a5, a6
   1269c:      	or	a5, a5, t0
   126a0:      	sw	a5, 0x0(t1)
   126a4:      	add	a1, a1, a4
   126a8:      	andi	a2, a2, 0x3
   126ac:      	add	a4, a3, a2
   126b0:      	bltu	a3, a4, 0x12538 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xa0>
   126b4:      	j	0x12550 <compiler_builtins::mem::memcpy::h2378d8e8f5ff767c+0xb8>

000126b8 <memset>:
   126b8:      	addi	sp, sp, -0x10
   126bc:      	sw	ra, 0xc(sp)
   126c0:      	sw	s0, 0x8(sp)
   126c4:      	addi	s0, sp, 0x10
   126c8:      	li	a3, 0x10
   126cc:      	bltu	a2, a3, 0x12750 <memset+0x98>
   126d0:      	neg	a3, a0
   126d4:      	andi	a3, a3, 0x3
   126d8:      	add	a4, a0, a3
   126dc:      	bgeu	a0, a4, 0x126f8 <memset+0x40>
   126e0:      	mv	a5, a3
   126e4:      	mv	a6, a0
   126e8:      	sb	a1, 0x0(a6)
   126ec:      	addi	a5, a5, -0x1
   126f0:      	addi	a6, a6, 0x1
   126f4:      	bnez	a5, 0x126e8 <memset+0x30>
   126f8:      	sub	a2, a2, a3
   126fc:      	andi	a3, a2, -0x4
   12700:      	add	a3, a4, a3
   12704:      	bgeu	a4, a3, 0x12724 <memset+0x6c>
   12708:      	andi	a5, a1, 0xff
   1270c:      	lui	a6, 0x1010
   12710:      	addi	a6, a6, 0x101
   12714:      	mul	a5, a5, a6
   12718:      	sw	a5, 0x0(a4)
   1271c:      	addi	a4, a4, 0x4
   12720:      	bltu	a4, a3, 0x12718 <memset+0x60>
   12724:      	andi	a2, a2, 0x3
   12728:      	add	a4, a3, a2
   1272c:      	bgeu	a3, a4, 0x12740 <memset+0x88>
   12730:      	sb	a1, 0x0(a3)
   12734:      	addi	a2, a2, -0x1
   12738:      	addi	a3, a3, 0x1
   1273c:      	bnez	a2, 0x12730 <memset+0x78>
   12740:      	lw	ra, 0xc(sp)
   12744:      	lw	s0, 0x8(sp)
   12748:      	addi	sp, sp, 0x10
   1274c:      	ret
   12750:      	mv	a3, a0
   12754:      	add	a4, a0, a2
   12758:      	bltu	a0, a4, 0x12730 <memset+0x78>
   1275c:      	j	0x12740 <memset+0x88>
