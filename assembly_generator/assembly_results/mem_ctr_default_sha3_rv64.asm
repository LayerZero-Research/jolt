
inline_benchmark:	file format elf64-littleriscv

Disassembly of section .text:

0000000000011384 <_start>:
   11384:      	addi	sp, sp, -0x1f0
   11386:      	sd	ra, 0x1e8(sp)
   11388:      	sd	s0, 0x1e0(sp)
   1138a:      	sd	s1, 0x1d8(sp)
   1138c:      	sd	s2, 0x1d0(sp)
   1138e:      	sd	s3, 0x1c8(sp)
   11390:      	sd	s4, 0x1c0(sp)
   11392:      	sd	s5, 0x1b8(sp)
   11394:      	sd	s6, 0x1b0(sp)
   11396:      	sd	s7, 0x1a8(sp)
   11398:      	sd	s8, 0x1a0(sp)
   1139a:      	sd	s9, 0x198(sp)
   1139c:      	sd	s10, 0x190(sp)
   1139e:      	sd	s11, 0x188(sp)
   113a0:      	addi	a0, sp, 0x8
   113a2:      	li	a2, 0xc8
   113a6:      	li	a1, 0x0
   113a8:      	auipc	ra, 0x0
   113ac:      	jalr	0x5e6(ra) <memset>
   113b0:      	addi	s0, sp, 0xd8
   113b2:      	li	a2, 0x89
   113b6:      	mv	a0, s0
   113b8:      	li	a1, 0x0
   113ba:      	auipc	ra, 0x0
   113be:      	jalr	0x5d4(ra) <memset>
   113c2:      	li	a0, 0x18
   113c4:      	sd	a0, 0xd0(sp)

00000000000113c6 <.Lpcrel_hi0>:
   113c6:      	auipc	a0, 0xfffff
   113ca:      	addi	a1, a0, -0x26e
   113ce:      	li	a2, 0xb
   113d0:      	li	s1, 0xb
   113d2:      	mv	a0, s0
   113d4:      	auipc	ra, 0x0
   113d8:      	jalr	0x63e(ra) <memcpy>
   113dc:      	sb	s1, 0x160(sp)
   113e0:      	addi	a0, sp, 0xe3
   113e4:      	li	a2, 0x7d
   113e8:      	li	a1, 0x0
   113ea:      	auipc	ra, 0x0
   113ee:      	jalr	0x5a4(ra) <memset>
   113f2:      	li	a6, 0x6
   113f4:      	lbu	t0, 0x15f(sp)
   113f8:      	ld	t2, 0x8(sp)
   113fa:      	ld	a7, 0x10(sp)
   113fc:      	ld	t1, 0x18(sp)
   113fe:      	ld	t4, 0x20(sp)
   11400:      	ld	t5, 0xf0(sp)
   11402:      	ld	t3, 0xf8(sp)
   11404:      	ld	t6, 0x100(sp)
   11406:      	ld	s2, 0x108(sp)
   11408:      	ld	s4, 0x28(sp)
   1140a:      	ld	s5, 0x30(sp)
   1140c:      	ld	s3, 0x38(sp)
   1140e:      	ld	s7, 0x40(sp)
   11410:      	ld	s8, 0x110(sp)
   11412:      	ld	s6, 0x118(sp)
   11414:      	ld	s9, 0x120(sp)
   11416:      	ld	s10, 0x128(sp)
   11418:      	ld	a1, 0xd0(sp)
   1141a:      	ld	a5, 0xd8(sp)
   1141c:      	sb	a6, 0xe3(sp)
   11420:      	ld	ra, 0xe0(sp)
   11422:      	ld	s11, 0xe8(sp)
   11424:      	xor	t2, t2, a5
   11428:      	xor	a6, t4, t5
   1142c:      	ld	a0, 0x48(sp)
   1142e:      	ld	a3, 0x50(sp)
   11430:      	ld	a4, 0x58(sp)
   11432:      	ld	s1, 0x60(sp)
   11434:      	xor	t5, s4, t3
   11438:      	xor	t4, s5, t6
   1143c:      	xor	t6, s3, s2
   11440:      	xor	t3, s7, s8
   11444:      	ld	a2, 0x130(sp)
   11446:      	ld	a5, 0x138(sp)
   11448:      	ld	s0, 0x140(sp)
   1144a:      	ld	s2, 0x148(sp)
   1144c:      	xor	s3, a0, s6
   11450:      	xor	a3, a3, s9
   11454:      	xor	a4, a4, s10
   11458:      	xor	s4, s1, a2
   1145c:      	ld	s1, 0x68(sp)
   1145e:      	ld	a0, 0x70(sp)
   11460:      	ld	a2, 0x78(sp)
   11462:      	ld	s5, 0x80(sp)
   11464:      	xor	a5, a5, s1
   11466:      	xor	a0, a0, s0
   11468:      	xor	a2, a2, s2
   1146c:      	ori	s1, t0, 0x80
   11470:      	sb	s1, 0x15f(sp)
   11474:      	sd	t5, 0x28(sp)
   11476:      	sd	t4, 0x30(sp)
   11478:      	sd	t6, 0x38(sp)
   1147a:      	sd	t3, 0x40(sp)
   1147c:      	sd	s3, 0x48(sp)
   1147e:      	sd	a3, 0x50(sp)
   11480:      	sd	a4, 0x58(sp)
   11482:      	sd	s4, 0x60(sp)
   11484:      	ld	a3, 0x150(sp)
   11486:      	xor	a4, a7, ra
   1148a:      	xor	s1, t1, s11
   1148e:      	ld	s0, 0x158(sp)
   11490:      	xor	a3, s5, a3
   11494:      	sd	t2, 0x8(sp)
   11496:      	sd	a4, 0x10(sp)
   11498:      	sd	s1, 0x18(sp)
   1149a:      	sd	a6, 0x20(sp)
   1149c:      	ld	a4, 0x88(sp)
   1149e:      	sd	a5, 0x68(sp)
   114a0:      	sd	a0, 0x70(sp)
   114a2:      	sd	a2, 0x78(sp)
   114a4:      	sd	a3, 0x80(sp)
   114a6:      	xor	a4, a4, s0
   114a8:      	sb	zero, 0x160(sp)
   114ac:      	sd	a4, 0x88(sp)
   114ae:      	addi	a0, sp, 0x8
   114b0:      	auipc	ra, 0x0
   114b4:      	jalr	0x36(ra) <keccak::p1600::h2013eaeec10b7a96>
   114b8:      	ld	a0, 0x8(sp)
   114ba:      	ld	a1, 0x10(sp)
   114bc:      	ld	a2, 0x18(sp)
   114be:      	ld	a3, 0x20(sp)
   114c0:      	sd	a0, 0x168(sp)
   114c2:      	sd	a1, 0x170(sp)
   114c4:      	sd	a2, 0x178(sp)
   114c6:      	sd	a3, 0x180(sp)
   114c8:      	addi	a0, sp, 0x168
   114ca:      	j	0x114ca <.Lpcrel_hi0+0x104>

00000000000114cc <core::panicking::panic_fmt::h609d02261d42ab3f>:
   114cc:      	addi	sp, sp, -0x10
   114ce:      	sd	ra, 0x8(sp)
   114d0:      	sd	s0, 0x0(sp)
   114d2:      	addi	s0, sp, 0x10
   114d4:      	j	0x114d4 <core::panicking::panic_fmt::h609d02261d42ab3f+0x8>

00000000000114d6 <core::panicking::panic::h37f2132fbb4345cc>:
   114d6:      	addi	sp, sp, -0x10
   114d8:      	sd	ra, 0x8(sp)
   114da:      	sd	s0, 0x0(sp)
   114dc:      	addi	s0, sp, 0x10
   114de:      	auipc	ra, 0x0
   114e2:      	jalr	-0x12(ra) <core::panicking::panic_fmt::h609d02261d42ab3f>

00000000000114e6 <keccak::p1600::h2013eaeec10b7a96>:
   114e6:      	addi	sp, sp, -0x110
   114e8:      	sd	ra, 0x108(sp)
   114ea:      	sd	s0, 0x100(sp)
   114ec:      	sd	s1, 0xf8(sp)
   114ee:      	sd	s2, 0xf0(sp)
   114f0:      	sd	s3, 0xe8(sp)
   114f2:      	sd	s4, 0xe0(sp)
   114f4:      	sd	s5, 0xd8(sp)
   114f6:      	sd	s6, 0xd0(sp)
   114f8:      	sd	s7, 0xc8(sp)
   114fa:      	sd	s8, 0xc0(sp)
   114fc:      	sd	s9, 0xb8(sp)
   114fe:      	sd	s10, 0xb0(sp)
   11500:      	sd	s11, 0xa8(sp)
   11502:      	li	a2, 0x18
   11504:      	bltu	a2, a1, 0x11986 <.Lpcrel_hi1+0x414>
   11508:      	beqz	a1, 0x11968 <.Lpcrel_hi1+0x3f6>
   1150c:      	slli	a3, a1, 0x3
   11510:      	ld	s4, 0x60(a0)
   11514:      	ld	t3, 0x68(a0)
   11518:      	ld	t6, 0x70(a0)
   1151c:      	ld	s3, 0x78(a0)
   11520:      	ld	s6, 0x40(a0)
   11524:      	ld	a1, 0x48(a0)
   11526:      	sd	a1, 0x80(sp)
   11528:      	ld	a6, 0x50(a0)
   1152c:      	ld	s10, 0x58(a0)
   11530:      	ld	a1, 0x20(a0)
   11532:      	sd	a1, 0xa0(sp)
   11534:      	ld	s8, 0x28(a0)
   11538:      	ld	s11, 0x30(a0)
   1153c:      	ld	a2, 0x38(a0)
   1153e:      	ld	a4, 0x0(a0)
   11540:      	ld	t4, 0x8(a0)
   11544:      	ld	a1, 0x10(a0)
   11546:      	sd	a1, 0x90(sp)
   11548:      	ld	a1, 0x18(a0)
   1154a:      	sd	a1, 0x98(sp)
   1154c:      	ld	a1, 0xa0(a0)
   1154e:      	sd	a1, 0x88(sp)
   11550:      	ld	t0, 0xa8(a0)
   11554:      	ld	a1, 0xb0(a0)
   11556:      	sd	a1, 0x78(sp)
   11558:      	ld	t1, 0xb8(a0)
   1155c:      	ld	s2, 0x80(a0)
   11560:      	ld	ra, 0x88(a0)
   11564:      	ld	t2, 0x90(a0)
   11568:      	ld	a7, 0x98(a0)
   1156c:      	sd	a0, 0x8(sp)
   1156e:      	ld	s9, 0xc0(a0)

0000000000011572 <.Lpcrel_hi1>:
   11572:      	auipc	s0, 0xfffff
   11576:      	addi	s0, s0, -0x40a
   1157a:      	sub	a3, s0, a3
   1157e:      	addi	a5, a3, 0xc0
   11582:      	addi	a0, s0, 0xc0
   11586:      	mv	s0, a6
   11588:      	sd	a0, 0x10(sp)
   1158a:      	sd	t3, 0x28(sp)
   1158c:      	sd	a4, 0x50(sp)
   1158e:      	sd	t6, 0x30(sp)
   11590:      	sd	s6, 0x38(sp)
   11592:      	sd	a5, 0x70(sp)
   11594:      	mv	s5, s8
   11596:      	xor	a3, s8, a4
   1159a:      	mv	a1, ra
   1159c:      	sd	ra, 0x58(sp)
   1159e:      	mv	ra, s2
   115a0:      	mv	s2, s0
   115a2:      	xor	s0, s3, s0
   115a6:      	xor	a0, s11, t4
   115aa:      	xor	t5, s0, a3
   115ae:      	xor	s0, ra, s10
   115b2:      	xor	a0, a0, s0
   115b4:      	mv	a3, t2
   115b6:      	sd	t2, 0x60(sp)
   115b8:      	sd	a7, 0x68(sp)
   115ba:      	mv	t2, a2
   115bc:      	ld	s8, 0x90(sp)
   115be:      	xor	s0, a2, s8
   115c2:      	mv	s7, s9
   115c4:      	sd	s9, 0x18(sp)
   115c6:      	sd	t1, 0x20(sp)
   115c8:      	xor	s1, a1, s4
   115cc:      	xor	s0, s0, s1
   115ce:      	ld	s9, 0x98(sp)
   115d0:      	xor	s1, s6, s9
   115d4:      	xor	a2, a3, t3
   115d8:      	xor	a2, a2, s1
   115da:      	ld	s6, 0x80(sp)
   115dc:      	ld	a1, 0xa0(sp)
   115de:      	xor	s1, s6, a1
   115e2:      	xor	a1, a7, t6
   115e6:      	xor	a1, a1, s1
   115e8:      	xor	a6, t0, a0
   115ec:      	srli	a0, a6, 0x3f
   115f0:      	slli	s1, a6, 0x1
   115f4:      	or	a0, a0, s1
   115f6:      	ld	a7, 0x78(sp)
   115f8:      	xor	a4, a7, s0
   115fc:      	srli	s1, a4, 0x3f
   11600:      	slli	s0, a4, 0x1
   11604:      	or	s0, s0, s1
   11606:      	xor	a3, t1, a2
   1160a:      	srli	a2, a3, 0x3f
   1160e:      	slli	s1, a3, 0x1
   11612:      	or	a2, a2, s1
   11614:      	xor	s1, s7, a1
   11618:      	srli	t3, s1, 0x3f
   1161c:      	slli	a1, s1, 0x1
   11620:      	or	t3, a1, t3
   11624:      	ld	s7, 0x88(sp)
   11626:      	xor	t6, s7, t5
   1162a:      	srli	t5, t6, 0x3f
   1162e:      	slli	a1, t6, 0x1
   11632:      	or	a1, a1, t5
   11636:      	xor	a0, a0, s1
   11638:      	xor	s1, s0, t6
   1163c:      	xor	a2, a2, a6
   11640:      	xor	a4, t3, a4
   11644:      	xor	a1, a1, a3
   11646:      	ld	a3, 0x50(sp)
   11648:      	xor	a3, a3, a0
   1164a:      	sd	a3, 0x50(sp)
   1164c:      	xor	a3, a0, s5
   11650:      	xor	t3, a0, s2
   11654:      	xor	t5, a0, s3
   11658:      	xor	a0, a0, s7
   1165c:      	sd	a0, 0x40(sp)
   1165e:      	xor	s0, s1, t4
   11662:      	xor	a0, s1, s11
   11666:      	sd	a0, 0x48(sp)
   11668:      	xor	t4, s1, s10
   1166c:      	xor	s7, s1, ra
   11670:      	xor	ra, s1, t0
   11674:      	xor	t0, a2, s8
   11678:      	xor	s1, a2, t2
   1167c:      	xor	t1, a2, s4
   11680:      	ld	a0, 0x58(sp)
   11682:      	xor	t6, a2, a0
   11686:      	xor	s2, a2, a7
   1168a:      	ld	a0, 0x0(a5)
   1168c:      	sd	a0, 0x90(sp)
   1168e:      	xor	s11, a4, s9
   11692:      	ld	a0, 0x38(sp)
   11694:      	xor	a6, a4, a0
   11698:      	ld	a2, 0x28(sp)
   1169a:      	xor	a2, a2, a4
   1169c:      	ld	a0, 0x60(sp)
   1169e:      	xor	s10, a4, a0
   116a2:      	ld	a0, 0x20(sp)
   116a4:      	xor	a4, a4, a0
   116a6:      	ld	a0, 0xa0(sp)
   116a8:      	xor	a0, a0, a1
   116aa:      	xor	s8, s6, a1
   116ae:      	ld	a5, 0x30(sp)
   116b0:      	xor	s3, a5, a1
   116b4:      	ld	a5, 0x68(sp)
   116b6:      	xor	a7, a5, a1
   116ba:      	ld	a5, 0x18(sp)
   116bc:      	xor	a1, a1, a5
   116be:      	srli	s4, s0, 0x3f
   116c2:      	slli	s0, s0, 0x1
   116c4:      	srli	s5, t3, 0x3d
   116c8:      	slli	t3, t3, 0x3
   116ca:      	or	a5, s0, s4
   116ce:      	sd	a5, 0x78(sp)
   116d0:      	srli	s0, s1, 0x3a
   116d4:      	slli	s1, s1, 0x6
   116d6:      	or	s4, t3, s5
   116da:      	srli	s9, t4, 0x36
   116de:      	slli	t4, t4, 0xa
   116e0:      	or	t3, s1, s0
   116e4:      	srli	s0, t6, 0x31
   116e8:      	slli	t6, t6, 0xf
   116ea:      	or	s9, t4, s9
   116ee:      	srli	t4, s10, 0x2b
   116f2:      	slli	s1, s10, 0x15
   116f6:      	or	a5, t6, s0
   116fa:      	sd	a5, 0x88(sp)
   116fc:      	srli	s0, s11, 0x24
   11700:      	slli	s11, s11, 0x1c
   11702:      	or	t6, s1, t4
   11706:      	srli	s1, a3, 0x1c
   1170a:      	slli	s6, a3, 0x24
   1170e:      	or	t4, s11, s0
   11712:      	srli	s0, s7, 0x13
   11716:      	slli	s7, s7, 0x2d
   11718:      	or	a3, s6, s1
   1171c:      	sd	a3, 0x58(sp)
   1171e:      	srli	s1, a6, 0x9
   11722:      	slli	a6, a6, 0x37
   11724:      	or	s6, s7, s0
   11728:      	srli	s0, ra, 0x3e
   1172c:      	slli	ra, ra, 0x2
   1172e:      	or	a3, a6, s1
   11732:      	sd	a3, 0x68(sp)
   11734:      	srli	a6, a1, 0x32
   11738:      	slli	a1, a1, 0xe
   1173a:      	or	a3, ra, s0
   1173e:      	sd	a3, 0x60(sp)
   11740:      	srli	ra, a0, 0x25
   11744:      	slli	a0, a0, 0x1b
   11746:      	or	s7, a1, a6
   1174a:      	srli	a1, t5, 0x17
   1174e:      	slli	a3, t5, 0x29
   11752:      	or	ra, a0, ra
   11756:      	srli	a0, a4, 0x8
   1175a:      	slli	a4, a4, 0x38
   1175c:      	or	s5, a3, a1
   11760:      	srli	a1, a7, 0x38
   11764:      	slli	a3, a7, 0x8
   11768:      	or	a7, a4, a0
   1176c:      	srli	a0, a2, 0x27
   11770:      	slli	a2, a2, 0x19
   11772:      	or	a3, a3, a1
   11774:      	srli	a1, t1, 0x15
   11778:      	slli	a4, t1, 0x2b
   1177c:      	or	a0, a0, a2
   1177e:      	srli	a2, t0, 0x2
   11782:      	slli	t2, t0, 0x3e
   11786:      	or	a4, a4, a1
   11788:      	ld	a5, 0x40(sp)
   1178a:      	srli	a1, a5, 0x2e
   1178e:      	slli	a5, a5, 0x12
   11790:      	or	t0, t2, a2
   11794:      	srli	a2, s3, 0x19
   11798:      	slli	s3, s3, 0x27
   1179a:      	or	t2, a5, a1
   1179e:      	srli	a1, s2, 0x3
   117a2:      	slli	a5, s2, 0x3d
   117a6:      	or	t1, s3, a2
   117aa:      	srli	s3, s8, 0x2c
   117ae:      	slli	a2, s8, 0x14
   117b2:      	or	a1, a1, a5
   117b4:      	ld	s2, 0x48(sp)
   117b6:      	srli	s8, s2, 0x14
   117ba:      	slli	s2, s2, 0x2c
   117bc:      	or	a2, a2, s3
   117c0:      	ld	a5, 0x50(sp)
   117c2:      	not	s3, a5
   117c6:      	ld	s1, 0x90(sp)
   117c8:      	xor	t5, s1, a5
   117cc:      	mv	s1, a5
   117ce:      	or	a5, s2, s8
   117d2:      	not	s0, s7
   117d6:      	and	s2, s1, s0
   117da:      	not	s0, t6
   117de:      	and	s3, a5, s3
   117e2:      	and	s0, s7, s0
   117e6:      	xor	s1, s3, s7
   117ea:      	sd	s1, 0xa0(sp)
   117ec:      	not	s1, a4
   117f0:      	and	s1, t6, s1
   117f4:      	xor	a6, s1, a5
   117f8:      	not	a5, a5
   117fc:      	and	s7, a4, a5
   11800:      	xor	a4, a4, s0
   11802:      	sd	a4, 0x90(sp)
   11804:      	not	a4, a2
   11808:      	xor	a5, t6, s2
   1180c:      	sd	a5, 0x98(sp)
   1180e:      	not	s1, a1
   11812:      	and	a4, s4, a4
   11816:      	and	s1, t4, s1
   1181a:      	xor	s8, a4, t4
   1181e:      	not	a4, t4
   11822:      	not	s0, s4
   11826:      	and	s0, s6, s0
   1182a:      	and	a4, a4, a2
   1182c:      	xor	s11, s0, a2
   11830:      	not	a2, s6
   11834:      	and	a2, a2, a1
   11836:      	xor	a5, s4, a2
   1183a:      	xor	s6, s1, s6
   1183e:      	not	a2, t3
   11842:      	xor	a1, a1, a4
   11844:      	sd	a1, 0x80(sp)
   11846:      	not	a1, t2
   1184a:      	and	a2, a2, a0
   1184c:      	ld	a4, 0x78(sp)
   1184e:      	and	a1, a1, a4
   11850:      	xor	s0, a2, a4
   11854:      	not	a2, a4
   11858:      	not	a4, a0
   1185c:      	and	a4, a4, a3
   1185e:      	and	a2, t3, a2
   11862:      	xor	s10, a4, t3
   11866:      	not	a4, a3
   1186a:      	and	a4, t2, a4
   1186e:      	xor	s4, a4, a0
   11872:      	xor	t3, a1, a3
   11876:      	ld	a4, 0x58(sp)
   11878:      	not	a0, a4
   1187c:      	xor	t6, t2, a2
   11880:      	not	a1, a7
   11884:      	and	a0, s9, a0
   11888:      	and	a1, ra, a1
   1188c:      	xor	s3, a0, ra
   11890:      	not	a0, ra
   11894:      	not	a2, s9
   11898:      	ld	a3, 0x88(sp)
   1189a:      	and	a2, a2, a3
   1189c:      	and	a0, a0, a4
   1189e:      	xor	s2, a4, a2
   118a2:      	not	a2, a3
   118a6:      	and	a2, a7, a2
   118aa:      	xor	ra, a2, s9
   118ae:      	xor	t2, a1, a3
   118b2:      	ld	a4, 0x68(sp)
   118b4:      	not	a1, a4
   118b8:      	xor	a7, a0, a7
   118bc:      	ld	a3, 0x60(sp)
   118be:      	not	a0, a3
   118c2:      	and	a1, t1, a1
   118c6:      	and	a0, t0, a0
   118ca:      	xor	a1, a1, t0
   118ce:      	sd	a1, 0x88(sp)
   118d0:      	not	a1, t0
   118d4:      	not	a2, t1
   118d8:      	and	a2, s5, a2
   118dc:      	and	a1, a1, a4
   118de:      	xor	t0, a2, a4
   118e2:      	not	a2, s5
   118e6:      	and	a2, a2, a3
   118e8:      	xor	a2, a2, t1
   118ec:      	sd	a2, 0x78(sp)
   118ee:      	mv	a2, a5
   118f0:      	xor	t1, s5, a0
   118f4:      	ld	a5, 0x70(sp)
   118f6:      	xor	s9, a1, a3
   118fa:      	addi	a5, a5, 0x8
   118fc:      	xor	a4, s7, t5
   11900:      	mv	t4, a6
   11902:      	ld	a0, 0x10(sp)
   11904:      	bne	a5, a0, 0x1158a <.Lpcrel_hi1+0x18>
   11908:      	ld	a0, 0x8(sp)
   1190a:      	sd	s4, 0x60(a0)
   1190e:      	sd	t3, 0x68(a0)
   11912:      	sd	t6, 0x70(a0)
   11916:      	sd	s3, 0x78(a0)
   1191a:      	sd	s6, 0x40(a0)
   1191e:      	ld	a1, 0x80(sp)
   11920:      	sd	a1, 0x48(a0)
   11922:      	sd	s0, 0x50(a0)
   11924:      	sd	s10, 0x58(a0)
   11928:      	ld	a1, 0xa0(sp)
   1192a:      	sd	a1, 0x20(a0)
   1192c:      	sd	s8, 0x28(a0)
   11930:      	sd	s11, 0x30(a0)
   11934:      	sd	a2, 0x38(a0)
   11936:      	sd	a4, 0x0(a0)
   11938:      	sd	t4, 0x8(a0)
   1193c:      	ld	a1, 0x90(sp)
   1193e:      	sd	a1, 0x10(a0)
   11940:      	ld	a1, 0x98(sp)
   11942:      	sd	a1, 0x18(a0)
   11944:      	ld	a1, 0x88(sp)
   11946:      	sd	a1, 0xa0(a0)
   11948:      	sd	t0, 0xa8(a0)
   1194c:      	ld	a1, 0x78(sp)
   1194e:      	sd	a1, 0xb0(a0)
   11950:      	sd	t1, 0xb8(a0)
   11954:      	sd	s2, 0x80(a0)
   11958:      	sd	ra, 0x88(a0)
   1195c:      	sd	t2, 0x90(a0)
   11960:      	sd	a7, 0x98(a0)
   11964:      	sd	s9, 0xc0(a0)
   11968:      	ld	ra, 0x108(sp)
   1196a:      	ld	s0, 0x100(sp)
   1196c:      	ld	s1, 0xf8(sp)
   1196e:      	ld	s2, 0xf0(sp)
   11970:      	ld	s3, 0xe8(sp)
   11972:      	ld	s4, 0xe0(sp)
   11974:      	ld	s5, 0xd8(sp)
   11976:      	ld	s6, 0xd0(sp)
   11978:      	ld	s7, 0xc8(sp)
   1197a:      	ld	s8, 0xc0(sp)
   1197c:      	ld	s9, 0xb8(sp)
   1197e:      	ld	s10, 0xb0(sp)
   11980:      	ld	s11, 0xa8(sp)
   11982:      	addi	sp, sp, 0x110
   11984:      	ret
   11986:      	auipc	ra, 0x0
   1198a:      	jalr	-0x4b0(ra) <core::panicking::panic::h37f2132fbb4345cc>

000000000001198e <memset>:
   1198e:      	addi	sp, sp, -0x10
   11990:      	sd	ra, 0x8(sp)
   11992:      	sd	s0, 0x0(sp)
   11994:      	addi	s0, sp, 0x10
   11996:      	li	a3, 0x10
   11998:      	bltu	a2, a3, 0x11a06 <memset+0x78>
   1199c:      	negw	a3, a0
   119a0:      	andi	a6, a3, 0x7
   119a4:      	add	a4, a0, a6
   119a8:      	bgeu	a0, a4, 0x119ba <memset+0x2c>
   119ac:      	mv	a5, a6
   119ae:      	mv	a3, a0
   119b0:      	sb	a1, 0x0(a3)
   119b4:      	addi	a5, a5, -0x1
   119b6:      	addi	a3, a3, 0x1
   119b8:      	bnez	a5, 0x119b0 <memset+0x22>
   119ba:      	sub	a2, a2, a6
   119be:      	andi	a3, a2, -0x8
   119c2:      	add	a3, a3, a4
   119c4:      	bgeu	a4, a3, 0x119ea <memset+0x5c>
   119c8:      	slli	a6, a1, 0x38
   119cc:      	lui	a5, 0x10101
   119d0:      	slli	a5, a5, 0x4
   119d2:      	addi	a5, a5, 0x100
   119d6:      	mulhu	a6, a6, a5
   119da:      	slli	a5, a6, 0x20
   119de:      	or	a5, a5, a6
   119e2:      	sd	a5, 0x0(a4)
   119e4:      	addi	a4, a4, 0x8
   119e6:      	bltu	a4, a3, 0x119e2 <memset+0x54>
   119ea:      	andi	a2, a2, 0x7
   119ec:      	add	a4, a3, a2
   119f0:      	bgeu	a3, a4, 0x119fe <memset+0x70>
   119f4:      	sb	a1, 0x0(a3)
   119f8:      	addi	a2, a2, -0x1
   119fa:      	addi	a3, a3, 0x1
   119fc:      	bnez	a2, 0x119f4 <memset+0x66>
   119fe:      	ld	ra, 0x8(sp)
   11a00:      	ld	s0, 0x0(sp)
   11a02:      	addi	sp, sp, 0x10
   11a04:      	ret
   11a06:      	mv	a3, a0
   11a08:      	add	a4, a0, a2
   11a0c:      	bltu	a0, a4, 0x119f4 <memset+0x66>
   11a10:      	j	0x119fe <memset+0x70>

0000000000011a12 <memcpy>:
   11a12:      	addi	sp, sp, -0x10
   11a14:      	sd	ra, 0x8(sp)
   11a16:      	sd	s0, 0x0(sp)
   11a18:      	addi	s0, sp, 0x10
   11a1a:      	ld	ra, 0x8(sp)
   11a1c:      	ld	s0, 0x0(sp)
   11a1e:      	addi	sp, sp, 0x10
   11a20:      	auipc	t1, 0x0
   11a24:      	jr	0x8(t1) <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6>

0000000000011a28 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6>:
   11a28:      	addi	sp, sp, -0x20
   11a2a:      	sd	ra, 0x18(sp)
   11a2c:      	sd	s0, 0x10(sp)
   11a2e:      	sd	s1, 0x8(sp)
   11a30:      	addi	s0, sp, 0x20
   11a32:      	li	a3, 0x10
   11a34:      	bltu	a2, a3, 0x11a98 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x70>
   11a38:      	negw	a3, a0
   11a3c:      	andi	a6, a3, 0x7
   11a40:      	add	t6, a0, a6
   11a44:      	bgeu	a0, t6, 0x11a5e <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x36>
   11a48:      	mv	a4, a6
   11a4a:      	mv	a3, a0
   11a4c:      	mv	a5, a1
   11a4e:      	lbu	a7, 0x0(a5)
   11a52:      	addi	a4, a4, -0x1
   11a54:      	sb	a7, 0x0(a3)
   11a58:      	addi	a3, a3, 0x1
   11a5a:      	addi	a5, a5, 0x1
   11a5c:      	bnez	a4, 0x11a4e <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x26>
   11a5e:      	add	a1, a1, a6
   11a60:      	sub	s1, a2, a6
   11a64:      	andi	a4, s1, -0x8
   11a68:      	andi	a6, a1, 0x7
   11a6c:      	add	a3, t6, a4
   11a70:      	bnez	a6, 0x11abc <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x94>
   11a74:      	bgeu	t6, a3, 0x11a88 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x60>
   11a78:      	mv	a5, a1
   11a7a:      	ld	a2, 0x0(a5)
   11a7c:      	sd	a2, 0x0(t6)
   11a80:      	addi	t6, t6, 0x8
   11a82:      	addi	a5, a5, 0x8
   11a84:      	bltu	t6, a3, 0x11a7a <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x52>
   11a88:      	add	a1, a1, a4
   11a8a:      	andi	a2, s1, 0x7
   11a8e:      	add	a4, a3, a2
   11a92:      	bltu	a3, a4, 0x11aa2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x7a>
   11a96:      	j	0x11ab2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x8a>
   11a98:      	mv	a3, a0
   11a9a:      	add	a4, a0, a2
   11a9e:      	bgeu	a0, a4, 0x11ab2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x8a>
   11aa2:      	lbu	a4, 0x0(a1)
   11aa6:      	addi	a2, a2, -0x1
   11aa8:      	sb	a4, 0x0(a3)
   11aac:      	addi	a3, a3, 0x1
   11aae:      	addi	a1, a1, 0x1
   11ab0:      	bnez	a2, 0x11aa2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x7a>
   11ab2:      	ld	ra, 0x18(sp)
   11ab4:      	ld	s0, 0x10(sp)
   11ab6:      	ld	s1, 0x8(sp)
   11ab8:      	addi	sp, sp, 0x20
   11aba:      	ret
   11abc:      	li	a7, 0x0
   11abe:      	li	a2, 0x8
   11ac0:      	sd	zero, -0x20(s0)
   11ac4:      	sub	t1, a2, a6
   11ac8:      	addi	a2, s0, -0x20
   11acc:      	andi	a5, t1, 0x1
   11ad0:      	or	t0, a2, a6
   11ad4:      	bnez	a5, 0x11b28 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x100>
   11ad6:      	andi	a2, t1, 0x2
   11ada:      	bnez	a2, 0x11b38 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x110>
   11adc:      	andi	a2, t1, 0x4
   11ae0:      	bnez	a2, 0x11b50 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x128>
   11ae2:      	ld	t4, -0x20(s0)
   11ae6:      	slli	a7, a6, 0x3
   11aea:      	addi	a2, t6, 0x8
   11aee:      	sub	t5, a1, a6
   11af2:      	bgeu	a2, a3, 0x11b70 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x148>
   11af6:      	negw	a2, a7
   11afa:      	andi	t2, a2, 0x38
   11afe:      	ld	t0, 0x8(t5)
   11b02:      	addi	t3, t5, 0x8
   11b06:      	srl	a2, t4, a7
   11b0a:      	addi	t1, t6, 0x8
   11b0e:      	sll	a5, t0, t2
   11b12:      	or	a2, a2, a5
   11b14:      	addi	a5, t6, 0x10
   11b18:      	sd	a2, 0x0(t6)
   11b1c:      	mv	t6, t1
   11b1e:      	mv	t5, t3
   11b20:      	mv	t4, t0
   11b22:      	bltu	a5, a3, 0x11afe <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xd6>
   11b26:      	j	0x11b76 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x14e>
   11b28:      	lbu	a2, 0x0(a1)
   11b2c:      	sb	a2, 0x0(t0)
   11b30:      	li	a7, 0x1
   11b32:      	andi	a2, t1, 0x2
   11b36:      	beqz	a2, 0x11adc <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xb4>
   11b38:      	add	a2, a1, a7
   11b3c:      	lh	a2, 0x0(a2)
   11b40:      	add	a5, t0, a7
   11b44:      	sh	a2, 0x0(a5)
   11b48:      	addi	a7, a7, 0x2
   11b4a:      	andi	a2, t1, 0x4
   11b4e:      	beqz	a2, 0x11ae2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xba>
   11b50:      	add	a2, a1, a7
   11b54:      	lw	a2, 0x0(a2)
   11b56:      	add	a7, a7, t0
   11b58:      	sw	a2, 0x0(a7)
   11b5c:      	ld	t4, -0x20(s0)
   11b60:      	slli	a7, a6, 0x3
   11b64:      	addi	a2, t6, 0x8
   11b68:      	sub	t5, a1, a6
   11b6c:      	bltu	a2, a3, 0x11af6 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xce>
   11b70:      	mv	t0, t4
   11b72:      	mv	t3, t5
   11b74:      	mv	t1, t6
   11b76:      	li	a5, 0x0
   11b78:      	addi	t2, t3, 0x8
   11b7c:      	li	a2, 0x4
   11b7e:      	sd	zero, -0x20(s0)
   11b82:      	bgeu	a6, a2, 0x11bcc <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x1a4>
   11b86:      	andi	a2, a1, 0x2
   11b8a:      	bnez	a2, 0x11bdc <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x1b4>
   11b8c:      	andi	a2, a1, 0x1
   11b90:      	beqz	a2, 0x11ba2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x17a>
   11b92:      	add	t2, t2, a5
   11b94:      	lbu	a6, 0x0(t2)
   11b98:      	addi	a2, s0, -0x20
   11b9c:      	or	a2, a2, a5
   11b9e:      	sb	a6, 0x0(a2)
   11ba2:      	ld	a6, -0x20(s0)
   11ba6:      	srl	a5, t0, a7
   11baa:      	negw	a2, a7
   11bae:      	andi	a2, a2, 0x38
   11bb2:      	sll	a2, a6, a2
   11bb6:      	or	a2, a2, a5
   11bb8:      	sd	a2, 0x0(t1)
   11bbc:      	add	a1, a1, a4
   11bbe:      	andi	a2, s1, 0x7
   11bc2:      	add	a4, a3, a2
   11bc6:      	bltu	a3, a4, 0x11aa2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x7a>
   11bca:      	j	0x11ab2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x8a>
   11bcc:      	lw	a2, 0x0(t2)
   11bd0:      	sw	a2, -0x20(s0)
   11bd4:      	li	a5, 0x4
   11bd6:      	andi	a2, a1, 0x2
   11bda:      	beqz	a2, 0x11b8c <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x164>
   11bdc:      	add	a2, t2, a5
   11be0:      	lh	a6, 0x0(a2)
   11be4:      	addi	a2, s0, -0x20
   11be8:      	or	a2, a2, a5
   11bea:      	sh	a6, 0x0(a2)
   11bee:      	addi	a5, a5, 0x2
   11bf0:      	andi	a2, a1, 0x1
   11bf4:      	bnez	a2, 0x11b92 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x16a>
   11bf6:      	j	0x11ba2 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x17a>
