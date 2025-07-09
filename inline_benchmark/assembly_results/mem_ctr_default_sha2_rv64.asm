
inline_benchmark:	file format elf64-littleriscv

Disassembly of section .text:

0000000000011228 <_start>:
   11228:      	addi	sp, sp, -0x180
   1122a:      	sd	ra, 0x178(sp)
   1122c:      	sd	s0, 0x170(sp)
   1122e:      	sd	s1, 0x168(sp)
   11230:      	sd	s2, 0x160(sp)
   11232:      	sd	s3, 0x158(sp)
   11234:      	sd	s4, 0x150(sp)
   11236:      	sd	s5, 0x148(sp)
   11238:      	sd	s6, 0x140(sp)
   1123a:      	sd	s7, 0x138(sp)
   1123c:      	sd	s8, 0x130(sp)
   1123e:      	sd	s9, 0x128(sp)
   11240:      	addi	s0, sp, 0x30
   11242:      	li	a2, 0x41
   11246:      	mv	a0, s0
   11248:      	li	a1, 0x0
   1124a:      	auipc	ra, 0x3
   1124e:      	jalr	0x562(ra) <memset>

0000000000011252 <.Lpcrel_hi0>:
   11252:      	auipc	a0, 0xfffff

0000000000011256 <.Lpcrel_hi1>:
   11256:      	auipc	a1, 0xfffff

000000000001125a <.Lpcrel_hi2>:
   1125a:      	auipc	a2, 0xfffff

000000000001125e <.Lpcrel_hi3>:
   1125e:      	auipc	a3, 0xfffff
   11262:      	ld	a0, -0xea(a0)
   11266:      	ld	a1, -0xf6(a1)
   1126a:      	ld	a2, -0x102(a2)
   1126e:      	ld	a3, -0xee(a3)
   11272:      	sd	a0, 0x8(sp)
   11274:      	sd	a1, 0x10(sp)
   11276:      	sd	a2, 0x18(sp)
   11278:      	sd	a3, 0x20(sp)
   1127a:      	sd	zero, 0x28(sp)

000000000001127c <.Lpcrel_hi4>:
   1127c:      	auipc	a0, 0xfffff
   11280:      	addi	a1, a0, -0x104
   11284:      	li	a2, 0xb
   11286:      	li	s1, 0xb
   11288:      	mv	a0, s0
   1128a:      	auipc	ra, 0x3
   1128e:      	jalr	0x5a6(ra) <memcpy>
   11292:      	sb	s1, 0x70(sp)
   11296:      	addi	a0, sp, 0x78
   11298:      	addi	a1, sp, 0x8
   1129a:      	li	a2, 0x70
   1129e:      	auipc	ra, 0x3
   112a2:      	jalr	0x592(ra) <memcpy>
   112a6:      	addi	s2, sp, 0xa0
   112aa:      	lbu	s1, 0xe0(sp)
   112ae:      	ld	a0, 0x98(sp)
   112b0:      	lui	a1, 0x10
   112b2:      	lui	a2, 0xff0
   112b6:      	addiw	a1, a1, -0x100
   112ba:      	slli	a3, a0, 0x9
   112be:      	slli	a4, s1, 0x3
   112c2:      	srli	a5, a0, 0x1f
   112c6:      	or	a4, a4, a3
   112c8:      	and	a5, a5, a1
   112ca:      	srli	a3, a3, 0x38
   112cc:      	or	a3, a3, a5
   112ce:      	srli	a5, a0, 0xf
   112d2:      	slli	a0, a0, 0x1
   112d4:      	and	a5, a5, a2
   112d6:      	srliw	a0, a0, 0x18
   112da:      	slli	a0, a0, 0x18
   112dc:      	or	a0, a0, a5
   112de:      	li	a6, 0x80
   112e2:      	or	a3, a3, a0
   112e4:      	li	a5, 0x3f
   112e8:      	and	a2, a2, a4
   112ea:      	and	a1, a1, a4
   112ec:      	srliw	a0, a4, 0x18
   112f0:      	slli	a0, a0, 0x20
   112f2:      	slli	a2, a2, 0x18
   112f4:      	or	a2, a2, a0
   112f6:      	slli	a0, s1, 0x3b
   112fa:      	slli	a1, a1, 0x28
   112fc:      	or	a1, a1, a0
   112fe:      	add	a0, s2, s1
   11302:      	or	a1, a1, a2
   11304:      	or	s0, a1, a3
   11308:      	sb	a6, 0x0(a0)
   1130c:      	beq	s1, a5, 0x1132a <.Lpcrel_hi4+0xae>
   11310:      	addi	a0, a0, 0x1
   11312:      	xori	a2, s1, 0x3f
   11316:      	li	a1, 0x0
   11318:      	auipc	ra, 0x3
   1131c:      	jalr	0x494(ra) <memset>
   11320:      	xori	a0, s1, 0x38
   11324:      	li	a1, 0x7
   11326:      	bltu	a1, a0, 0x1134c <.Lpcrel_hi4+0xd0>
   1132a:      	addi	a0, sp, 0x78
   1132c:      	mv	a1, s2
   1132e:      	auipc	ra, 0x0
   11332:      	jalr	0x152(ra) <sha2::sha256::compress256::hdd2cd6517babac1f>
   11336:      	sd	zero, 0xe8(sp)
   11338:      	sd	zero, 0xf0(sp)
   1133a:      	sd	zero, 0xf8(sp)
   1133c:      	sd	zero, 0x100(sp)
   1133e:      	sd	zero, 0x108(sp)
   11340:      	sd	zero, 0x110(sp)
   11342:      	sd	zero, 0x118(sp)
   11344:      	sd	s0, 0x120(sp)
   11346:      	addi	a0, sp, 0x78
   11348:      	addi	a1, sp, 0xe8
   1134a:      	j	0x11352 <.Lpcrel_hi4+0xd6>
   1134c:      	sd	s0, 0xd8(sp)
   1134e:      	addi	a0, sp, 0x78
   11350:      	mv	a1, s2
   11352:      	auipc	ra, 0x0
   11356:      	jalr	0x12e(ra) <sha2::sha256::compress256::hdd2cd6517babac1f>
   1135a:      	lui	a6, 0x10
   1135c:      	lw	a7, 0x78(sp)
   1135e:      	lw	t0, 0x7c(sp)
   11360:      	lw	a4, 0x80(sp)
   11362:      	lw	a5, 0x84(sp)
   11364:      	lw	s1, 0x88(sp)
   11366:      	lw	a3, 0x8c(sp)
   11368:      	lw	s9, 0x90(sp)
   1136a:      	lw	s8, 0x94(sp)
   1136c:      	addi	a0, a6, -0x100
   11370:      	srli	a2, a7, 0x8
   11374:      	srliw	a6, a7, 0x18
   11378:      	and	t1, a7, a0
   1137c:      	slli	a7, a7, 0x18
   1137e:      	srli	t2, t0, 0x8
   11382:      	srliw	t3, t0, 0x18
   11386:      	and	t4, t0, a0
   1138a:      	slli	t0, t0, 0x18
   1138c:      	srli	t5, a4, 0x8
   11390:      	srliw	t6, a4, 0x18
   11394:      	and	s2, a4, a0
   11398:      	slli	s4, a4, 0x18
   1139c:      	and	a2, a2, a0
   1139e:      	or	a6, a2, a6
   113a2:      	srli	s3, a5, 0x8
   113a6:      	slli	t1, t1, 0x8
   113a8:      	or	a7, a7, t1
   113ac:      	srliw	s5, a5, 0x18
   113b0:      	and	a4, t2, a0
   113b4:      	or	t1, a4, t3
   113b8:      	and	t2, a5, a0
   113bc:      	slli	s6, a5, 0x18
   113c0:      	slli	t4, t4, 0x8
   113c2:      	or	t0, t0, t4
   113c6:      	srli	s7, s1, 0x8
   113ca:      	and	a4, t5, a0
   113ce:      	or	t3, a4, t6
   113d2:      	srliw	t6, s1, 0x18
   113d6:      	slli	s2, s2, 0x8
   113d8:      	or	t4, s4, s2
   113dc:      	and	s2, s1, a0
   113e0:      	slli	s1, s1, 0x18
   113e2:      	and	a2, s3, a0
   113e6:      	or	t5, a2, s5
   113ea:      	srli	a2, a3, 0x8
   113ee:      	slli	t2, t2, 0x8
   113f0:      	or	t2, s6, t2
   113f4:      	srliw	a4, a3, 0x18
   113f8:      	and	a5, s7, a0
   113fc:      	or	t6, a5, t6
   11400:      	and	s0, a3, a0
   11404:      	slli	a3, a3, 0x18
   11406:      	slli	s2, s2, 0x8
   11408:      	or	s2, s1, s2
   1140c:      	srli	a1, s9, 0x8
   11410:      	and	a2, a2, a0
   11412:      	or	s3, a2, a4
   11416:      	srliw	a4, s9, 0x18
   1141a:      	slli	s0, s0, 0x8
   1141c:      	or	s4, a3, s0
   11420:      	and	s0, s9, a0
   11424:      	slli	s9, s9, 0x18
   11426:      	and	a1, a1, a0
   11428:      	or	s5, a1, a4
   1142c:      	srli	a4, s8, 0x8
   11430:      	slli	s0, s0, 0x8
   11432:      	or	s6, s9, s0
   11436:      	srliw	a5, s8, 0x18
   1143a:      	and	a4, a4, a0
   1143c:      	or	s7, a4, a5
   11440:      	and	a0, s8, a0
   11444:      	slli	s8, s8, 0x18
   11446:      	slli	a0, a0, 0x8
   11448:      	or	a0, s8, a0
   1144c:      	or	a5, a7, a6
   11450:      	or	s1, t0, t1
   11454:      	or	a2, t4, t3
   11458:      	or	a3, t2, t5
   1145c:      	or	a1, s2, t6
   11460:      	or	s0, s4, s3
   11464:      	or	a4, s6, s5
   11468:      	or	a0, a0, s7
   1146c:      	sw	a5, 0x78(sp)
   1146e:      	sw	s1, 0x7c(sp)
   11470:      	sw	a2, 0x80(sp)
   11472:      	sw	a3, 0x84(sp)
   11474:      	sw	a1, 0x88(sp)
   11476:      	sw	s0, 0x8c(sp)
   11478:      	sw	a4, 0x90(sp)
   1147a:      	sw	a0, 0x94(sp)
   1147c:      	addi	a0, sp, 0x78
   1147e:      	j	0x1147e <.Lpcrel_hi4+0x202>

0000000000011480 <sha2::sha256::compress256::hdd2cd6517babac1f>:
   11480:      	addi	sp, sp, -0x240
   11484:      	sd	ra, 0x238(sp)
   11488:      	sd	s0, 0x230(sp)
   1148c:      	sd	s1, 0x228(sp)
   11490:      	sd	s2, 0x220(sp)
   11494:      	sd	s3, 0x218(sp)
   11498:      	sd	s4, 0x210(sp)
   1149c:      	sd	s5, 0x208(sp)
   114a0:      	sd	s6, 0x200(sp)
   114a4:      	sd	s7, 0x1f8(sp)
   114a6:      	sd	s8, 0x1f0(sp)
   114a8:      	sd	s9, 0x1e8(sp)
   114aa:      	sd	s10, 0x1e0(sp)
   114ac:      	sd	s11, 0x1d8(sp)
   114ae:      	lbu	a6, 0x3c(a1)
   114b2:      	lbu	a7, 0x3d(a1)
   114b6:      	lbu	s4, 0x3e(a1)
   114ba:      	lb	s5, 0x3f(a1)
   114be:      	lbu	t2, 0x38(a1)
   114c2:      	lbu	t3, 0x39(a1)
   114c6:      	lbu	a2, 0x3a(a1)
   114ca:      	sd	a2, 0x1d0(sp)
   114cc:      	lb	ra, 0x3b(a1)
   114d0:      	lbu	t0, 0x20(a1)
   114d4:      	lbu	t1, 0x21(a1)
   114d8:      	lbu	a2, 0x22(a1)
   114dc:      	sd	a2, 0x1c8(sp)
   114de:      	lb	a2, 0x23(a1)
   114e2:      	sd	a2, 0x1c0(sp)
   114e4:      	lbu	t5, 0x1c(a1)
   114e8:      	lbu	s2, 0x1d(a1)
   114ec:      	lbu	a3, 0x1e(a1)
   114f0:      	sd	a3, 0x1b8(sp)
   114f2:      	lb	a3, 0x1f(a1)
   114f6:      	sd	a3, 0x198(sp)
   114f8:      	lbu	a3, 0x4(a1)
   114fc:      	lbu	a4, 0x5(a1)
   11500:      	lbu	a5, 0x6(a1)
   11504:      	sd	a5, 0x190(sp)
   11506:      	lb	a5, 0x7(a1)
   1150a:      	sd	a5, 0x188(sp)
   1150c:      	lbu	t4, 0x0(a1)
   11510:      	lbu	t6, 0x1(a1)
   11514:      	lbu	s1, 0x2(a1)
   11518:      	lb	s0, 0x3(a1)
   1151c:      	slli	a7, a7, 0x10
   1151e:      	slli	a6, a6, 0x18
   11520:      	slli	t3, t3, 0x10
   11522:      	slli	t2, t2, 0x18
   11524:      	or	s3, a6, a7
   11528:      	or	a2, t2, t3
   1152c:      	sd	a2, 0x168(sp)
   1152e:      	lbu	a7, 0x24(a1)
   11532:      	lbu	t2, 0x25(a1)
   11536:      	lbu	a2, 0x26(a1)
   1153a:      	sd	a2, 0x180(sp)
   1153c:      	lb	a2, 0x27(a1)
   11540:      	sd	a2, 0x178(sp)
   11542:      	slli	t1, t1, 0x10
   11544:      	slli	t0, t0, 0x18
   11546:      	slli	a2, s2, 0x10
   1154a:      	slli	t5, t5, 0x18
   1154c:      	slli	a4, a4, 0x10
   1154e:      	slli	a3, a3, 0x18
   11550:      	andi	s0, s0, 0xff
   11554:      	slli	s1, s1, 0x8
   11556:      	or	a5, t0, t1
   1155a:      	sd	a5, 0x150(sp)
   1155c:      	or	a2, t5, a2
   11560:      	sd	a2, 0x158(sp)
   11562:      	or	t1, a3, a4
   11566:      	or	s0, s0, s1
   11568:      	sd	s0, 0x160(sp)
   1156a:      	lbu	a2, 0x18(a1)
   1156e:      	lbu	a3, 0x19(a1)
   11572:      	lbu	a4, 0x1a(a1)
   11576:      	sd	a4, 0x148(sp)
   11578:      	lb	a4, 0x1b(a1)
   1157c:      	sd	a4, 0x140(sp)
   1157e:      	slli	t6, t6, 0x10
   11580:      	slli	t4, t4, 0x18
   11582:      	slli	a5, t2, 0x10
   11586:      	slli	a7, a7, 0x18
   11588:      	slli	a3, a3, 0x10
   1158a:      	slli	a2, a2, 0x18
   1158c:      	or	a4, t4, t6
   11590:      	sd	a4, 0x130(sp)
   11592:      	or	a4, a7, a5
   11596:      	sd	a4, 0x138(sp)
   11598:      	lbu	a5, 0x14(a1)
   1159c:      	lbu	a4, 0x15(a1)
   115a0:      	or	t0, a2, a3
   115a4:      	lbu	a2, 0x16(a1)
   115a8:      	sd	a2, 0x128(sp)
   115aa:      	lb	a2, 0x17(a1)
   115ae:      	sd	a2, 0xf0(sp)
   115b0:      	slli	a4, a4, 0x10
   115b2:      	slli	a5, a5, 0x18
   115b4:      	or	t2, a5, a4
   115b8:      	lbu	a2, 0x11(a1)
   115bc:      	lbu	a3, 0x10(a1)
   115c0:      	lbu	a4, 0x12(a1)
   115c4:      	sd	a4, 0xe8(sp)
   115c6:      	lb	a4, 0x13(a1)
   115ca:      	sd	a4, 0xe0(sp)
   115cc:      	slli	a2, a2, 0x10
   115ce:      	slli	a3, a3, 0x18
   115d0:      	or	a2, a2, a3
   115d2:      	sd	a2, 0xd8(sp)
   115d4:      	lbu	a2, 0xd(a1)
   115d8:      	lbu	a3, 0xc(a1)
   115dc:      	lbu	a4, 0xe(a1)
   115e0:      	sd	a4, 0xd0(sp)
   115e2:      	lb	a4, 0xf(a1)
   115e6:      	sd	a4, 0xc8(sp)
   115e8:      	slli	a2, a2, 0x10
   115ea:      	slli	a3, a3, 0x18
   115ec:      	or	a2, a2, a3
   115ee:      	sd	a2, 0x60(sp)
   115f0:      	lbu	a3, 0x31(a1)
   115f4:      	lbu	a4, 0x30(a1)
   115f8:      	lbu	a2, 0x32(a1)
   115fc:      	sd	a2, 0xb8(sp)
   115fe:      	lb	a2, 0x33(a1)
   11602:      	sd	a2, 0xb0(sp)
   11604:      	slli	a3, a3, 0x10
   11606:      	slli	a4, a4, 0x18
   11608:      	or	a3, a3, a4
   1160a:      	sd	a3, 0xa8(sp)
   1160c:      	lbu	a3, 0x9(a1)
   11610:      	lbu	a4, 0x8(a1)
   11614:      	lbu	a2, 0xa(a1)
   11618:      	sd	a2, 0xa0(sp)
   1161a:      	lb	a2, 0xb(a1)
   1161e:      	sd	a2, 0x98(sp)
   11620:      	slli	a3, a3, 0x10
   11622:      	slli	a4, a4, 0x18
   11624:      	or	a3, a3, a4
   11626:      	sd	a3, 0x88(sp)
   11628:      	lbu	a3, 0x29(a1)
   1162c:      	lbu	a4, 0x28(a1)
   11630:      	lbu	a2, 0x2a(a1)
   11634:      	sd	a2, 0x80(sp)
   11636:      	lb	s11, 0x2b(a1)
   1163a:      	slli	a3, a3, 0x10
   1163c:      	slli	a4, a4, 0x18
   1163e:      	or	a3, a3, a4
   11640:      	sd	a3, 0x78(sp)
   11642:      	lbu	a4, 0x35(a1)
   11646:      	lbu	a5, 0x34(a1)
   1164a:      	lbu	a2, 0x36(a1)
   1164e:      	sd	a2, 0x70(sp)
   11650:      	lb	a3, 0x37(a1)
   11654:      	slli	a4, a4, 0x10
   11656:      	slli	a5, a5, 0x18
   11658:      	or	a4, a4, a5
   1165a:      	sd	a4, 0x68(sp)
   1165c:      	lbu	a4, 0x2d(a1)
   11660:      	lbu	a5, 0x2c(a1)
   11664:      	lbu	s9, 0x2e(a1)
   11668:      	lb	s6, 0x2f(a1)
   1166c:      	slli	a4, a4, 0x10
   1166e:      	slli	a5, a5, 0x18
   11670:      	or	s8, a5, a4
   11674:      	lw	a4, 0x10(a0)
   11676:      	lw	a1, 0x14(a0)
   11678:      	sd	a1, 0x1a8(sp)
   1167a:      	lw	a1, 0x18(a0)
   1167c:      	sd	a1, 0x1b0(sp)
   1167e:      	lw	a2, 0x1c(a0)
   11680:      	sd	a2, 0x118(sp)
   11682:      	mv	a1, a4
   11684:      	sd	a4, 0x1a0(sp)
   11686:      	srliw	a4, a4, 0x6
   1168a:      	slli	a5, a1, 0x1a
   1168e:      	or	a4, a4, a5
   11690:      	sd	a4, 0x58(sp)
   11692:      	srliw	a4, a1, 0xb
   11696:      	slli	a5, a1, 0x15
   1169a:      	or	a4, a4, a5
   1169c:      	sd	a4, 0x48(sp)
   1169e:      	srliw	a4, a1, 0x19
   116a2:      	slli	a5, a1, 0x7
   116a6:      	or	a4, a4, a5
   116a8:      	sd	a4, 0x50(sp)
   116aa:      	sd	a0, 0x90(sp)
   116ac:      	lw	a2, 0x0(a0)
   116ae:      	lw	s0, 0x4(a0)
   116b0:      	lw	t3, 0x8(a0)
   116b4:      	lw	a0, 0xc(a0)
   116b6:      	sd	a0, 0x110(sp)
   116b8:      	srliw	a4, a2, 0x2
   116bc:      	slli	a5, a2, 0x1e
   116c0:      	or	s10, a5, a4
   116c4:      	srliw	a4, a2, 0xd
   116c8:      	slli	a5, a2, 0x13
   116cc:      	or	s7, a5, a4
   116d0:      	srliw	a4, a2, 0x16
   116d4:      	slli	s1, a2, 0xa
   116d8:      	mv	a5, a2
   116da:      	sd	a2, 0xf8(sp)
   116dc:      	or	s2, s1, a4
   116e0:      	andi	a6, s5, 0xff
   116e4:      	slli	a4, s4, 0x8
   116e8:      	or	a4, a4, a6
   116ec:      	or	a1, s3, a4
   116f0:      	andi	a0, ra, 0xff
   116f4:      	sd	a0, 0x40(sp)
   116f6:      	ld	a4, 0x1d0(sp)
   116f8:      	slli	a4, a4, 0x8
   116fa:      	or	a4, a4, a0
   116fc:      	ld	a0, 0x168(sp)
   116fe:      	or	s4, a0, a4
   11702:      	lbu	t5, 0x1c0(sp)
   11706:      	ld	a4, 0x1c8(sp)
   11708:      	slli	a4, a4, 0x8
   1170a:      	or	a4, a4, t5
   1170e:      	ld	a0, 0x150(sp)
   11710:      	or	a7, a0, a4
   11714:      	lbu	a0, 0x198(sp)
   11718:      	sd	a0, 0x150(sp)
   1171a:      	ld	a4, 0x1b8(sp)
   1171c:      	slli	a4, a4, 0x8
   1171e:      	or	a4, a4, a0
   11720:      	ld	a0, 0x158(sp)
   11722:      	or	a0, a0, a4
   11724:      	sd	a0, 0x1b8(sp)
   11726:      	lbu	t4, 0x188(sp)
   1172a:      	ld	a4, 0x190(sp)
   1172c:      	slli	a4, a4, 0x8
   1172e:      	or	a4, a4, t4
   11732:      	or	a4, t1, a4
   11736:      	ld	a0, 0x160(sp)
   11738:      	ld	a2, 0x130(sp)
   1173a:      	or	a0, a0, a2
   1173c:      	sd	a0, 0x198(sp)
   1173e:      	lbu	s3, 0x178(sp)
   11742:      	ld	a2, 0x180(sp)
   11744:      	slli	a2, a2, 0x8
   11746:      	or	a2, a2, s3
   1174a:      	ld	a0, 0x138(sp)
   1174c:      	or	a0, a0, a2
   1174e:      	sd	a0, 0x1c0(sp)
   11750:      	lbu	t1, 0x140(sp)
   11754:      	ld	a2, 0x148(sp)
   11756:      	slli	a2, a2, 0x8
   11758:      	or	a2, a2, t1
   1175c:      	or	s5, t0, a2
   11760:      	lbu	a0, 0xf0(sp)
   11764:      	sd	a0, 0x180(sp)
   11766:      	ld	a2, 0x128(sp)
   11768:      	slli	a2, a2, 0x8
   1176a:      	or	a2, a2, a0
   1176c:      	or	a0, t2, a2
   11770:      	sd	a0, 0x168(sp)
   11772:      	lbu	t2, 0xe0(sp)
   11776:      	ld	a2, 0xe8(sp)
   11778:      	slli	a2, a2, 0x8
   1177a:      	or	a2, a2, t2
   1177e:      	ld	a0, 0xd8(sp)
   11780:      	or	t6, a0, a2
   11784:      	lbu	ra, 0xc8(sp)
   11788:      	ld	a2, 0xd0(sp)
   1178a:      	slli	a2, a2, 0x8
   1178c:      	or	a2, a2, ra
   11790:      	ld	a0, 0x60(sp)
   11792:      	or	a0, a0, a2
   11794:      	sd	a0, 0x160(sp)
   11796:      	lbu	a2, 0xb0(sp)
   1179a:      	sd	a2, 0x190(sp)
   1179c:      	ld	a0, 0xb8(sp)
   1179e:      	slli	a0, a0, 0x8
   117a0:      	or	a0, a0, a2
   117a2:      	ld	a2, 0xa8(sp)
   117a4:      	or	a0, a0, a2
   117a6:      	sd	a0, 0x1d0(sp)
   117a8:      	lbu	a0, 0x98(sp)
   117ac:      	ld	a2, 0xa0(sp)
   117ae:      	slli	a2, a2, 0x8
   117b0:      	or	a2, a2, a0
   117b2:      	ld	s1, 0x88(sp)
   117b4:      	or	t0, s1, a2
   117b8:      	andi	s1, s11, 0xff
   117bc:      	sd	s1, 0x188(sp)
   117be:      	ld	a2, 0x80(sp)
   117c0:      	slli	a2, a2, 0x8
   117c2:      	or	a2, a2, s1
   117c4:      	ld	s1, 0x78(sp)
   117c6:      	or	a2, a2, s1
   117c8:      	sd	a2, 0x1c8(sp)
   117ca:      	andi	a3, a3, 0xff
   117ce:      	ld	a2, 0x70(sp)
   117d0:      	slli	a2, a2, 0x8
   117d2:      	or	a2, a2, a3
   117d4:      	ld	s1, 0x68(sp)
   117d6:      	or	s11, s1, a2
   117da:      	andi	s6, s6, 0xff
   117de:      	slli	s9, s9, 0x8
   117e0:      	or	a2, s9, s6
   117e4:      	or	s8, s8, a2
   117e8:      	ld	a2, 0x58(sp)
   117ea:      	ld	s1, 0x48(sp)
   117ec:      	xor	s9, a2, s1
   117f0:      	sd	t3, 0x108(sp)
   117f2:      	mv	a2, s0
   117f4:      	sd	s0, 0x100(sp)
   117f6:      	xor	s0, t3, s0
   117fa:      	and	s0, s0, a5
   117fc:      	and	s1, t3, a2
   11800:      	xor	t3, s0, s1
   11804:      	xor	s0, s10, s7
   11808:      	ld	a2, 0x50(sp)
   1180a:      	xor	a2, s9, a2
   1180e:      	sd	a2, 0x138(sp)
   11810:      	xor	a2, s0, s2
   11814:      	sd	a2, 0x130(sp)
   11816:      	slli	a5, a6, 0x19
   1181a:      	srliw	a2, a1, 0x7
   1181e:      	or	a2, a2, a5
   11820:      	sd	a2, 0x128(sp)
   11822:      	srliw	a2, a1, 0x12
   11826:      	slli	a5, a1, 0xe
   1182a:      	or	a2, a2, a5
   1182c:      	sd	a2, 0xf0(sp)
   1182e:      	slli	t5, t5, 0x19
   11830:      	sd	a7, 0x120(sp)
   11832:      	srliw	a2, a7, 0x7
   11836:      	or	a2, t5, a2
   1183a:      	sd	a2, 0xe8(sp)
   1183c:      	srliw	a2, a7, 0x12
   11840:      	slli	a5, a7, 0xe
   11844:      	or	a2, a2, a5
   11846:      	sd	a2, 0xe0(sp)
   11848:      	slli	t4, t4, 0x19
   1184a:      	sd	a4, 0x38(sp)
   1184c:      	srliw	a2, a4, 0x7
   11850:      	or	a2, t4, a2
   11854:      	sd	a2, 0xd8(sp)
   11856:      	srliw	a2, a4, 0x12
   1185a:      	slli	a5, a4, 0xe
   1185e:      	or	a2, a2, a5
   11860:      	sd	a2, 0xd0(sp)
   11862:      	srliw	a2, s4, 0x11
   11866:      	slli	a5, s4, 0xf
   1186a:      	or	a2, a2, a5
   1186c:      	sd	a2, 0xc8(sp)
   1186e:      	srliw	a2, s4, 0x13
   11872:      	slli	a5, s4, 0xd
   11876:      	or	a2, a2, a5
   11878:      	sd	a2, 0xb8(sp)
   1187a:      	slli	t1, t1, 0x19
   1187c:      	sd	s5, 0x140(sp)
   1187e:      	srliw	a2, s5, 0x7
   11882:      	or	t1, t1, a2
   11886:      	srliw	a2, s5, 0x12
   1188a:      	slli	s1, s5, 0xe
   1188e:      	or	a2, a2, s1
   11890:      	sd	a2, 0xb0(sp)
   11892:      	slli	t2, t2, 0x19
   11894:      	sd	t6, 0x158(sp)
   11896:      	srliw	a2, t6, 0x7
   1189a:      	or	a2, t2, a2
   1189e:      	sd	a2, 0xa8(sp)
   118a0:      	srliw	a2, t6, 0x12
   118a4:      	slli	a5, t6, 0xe
   118a8:      	or	a2, a2, a5
   118aa:      	sd	a2, 0xa0(sp)
   118ac:      	slli	a0, a0, 0x19
   118ae:      	sd	t0, 0x98(sp)
   118b0:      	srliw	a2, t0, 0x7
   118b4:      	or	a0, a0, a2
   118b6:      	sd	a0, 0x88(sp)
   118b8:      	srliw	a0, t0, 0x12
   118bc:      	slli	a2, t0, 0xe
   118c0:      	or	a0, a0, a2
   118c2:      	sd	a0, 0x80(sp)
   118c4:      	sd	a1, 0x170(sp)
   118c6:      	srliw	a0, a1, 0x11
   118ca:      	slli	a2, a1, 0xf
   118ce:      	or	a0, a0, a2
   118d0:      	sd	a0, 0x78(sp)
   118d2:      	srliw	a0, a1, 0x13
   118d6:      	slli	a5, a1, 0xd
   118da:      	or	a0, a0, a5
   118dc:      	sd	a0, 0x70(sp)
   118de:      	slli	a3, a3, 0x19
   118e0:      	sd	s11, 0x148(sp)
   118e2:      	srliw	a0, s11, 0x7
   118e6:      	or	a0, a0, a3
   118e8:      	sd	a0, 0x68(sp)
   118ea:      	srliw	a0, s11, 0x12
   118ee:      	slli	a2, s11, 0xe
   118f2:      	or	s9, a2, a0
   118f6:      	slli	a1, s6, 0x19
   118fa:      	sd	s8, 0x178(sp)
   118fc:      	srliw	a0, s8, 0x7
   11900:      	or	a0, a0, a1
   11902:      	sd	a0, 0x60(sp)
   11904:      	srliw	a0, s8, 0x12
   11908:      	slli	a1, s8, 0xe
   1190c:      	or	s7, a1, a0
   11910:      	slli	a4, s3, 0x19
   11914:      	ld	s10, 0x1c0(sp)
   11916:      	srliw	a0, s10, 0x7
   1191a:      	or	t4, a4, a0
   1191e:      	srliw	a4, s10, 0x12
   11922:      	slli	s10, s10, 0xe
   11924:      	or	s5, s10, a4
   11928:      	ld	t6, 0x150(sp)
   1192a:      	slli	t6, t6, 0x19
   1192c:      	ld	s0, 0x1b8(sp)
   1192e:      	srliw	s1, s0, 0x7
   11932:      	or	s10, t6, s1
   11936:      	srliw	s1, s0, 0x12
   1193a:      	slli	s0, s0, 0xe
   1193c:      	or	t2, s0, s1
   11940:      	ld	a7, 0x180(sp)
   11942:      	slli	a7, a7, 0x19
   11944:      	ld	a5, 0x168(sp)
   11946:      	srliw	s1, a5, 0x7
   1194a:      	or	a7, a7, s1
   1194e:      	srliw	s1, a5, 0x12
   11952:      	slli	a5, a5, 0xe
   11954:      	or	t5, a5, s1
   11958:      	slli	a6, ra, 0x19
   1195c:      	ld	a3, 0x160(sp)
   1195e:      	srliw	s1, a3, 0x7
   11962:      	or	s8, a6, s1
   11966:      	srliw	s1, a3, 0x12
   1196a:      	slli	a3, a3, 0xe
   1196c:      	or	s11, a3, s1
   11970:      	ld	ra, 0x40(sp)
   11972:      	slli	ra, ra, 0x19
   11974:      	mv	t6, s4
   11976:      	srliw	s1, s4, 0x7
   1197a:      	or	ra, ra, s1
   1197e:      	srliw	s1, s4, 0x12
   11982:      	slli	a2, s4, 0xe
   11986:      	or	t0, a2, s1
   1198a:      	ld	s2, 0x190(sp)
   1198c:      	slli	s2, s2, 0x19
   1198e:      	ld	a1, 0x1d0(sp)
   11990:      	srliw	s1, a1, 0x7
   11994:      	or	s2, s2, s1
   11998:      	srliw	s1, a1, 0x12
   1199c:      	slli	a1, a1, 0xe
   1199e:      	or	a6, a1, s1
   119a2:      	ld	s3, 0x188(sp)
   119a4:      	slli	s3, s3, 0x19
   119a6:      	ld	a0, 0x1c8(sp)
   119a8:      	srliw	s1, a0, 0x7
   119ac:      	or	s3, s3, s1
   119b0:      	srliw	s1, a0, 0x12
   119b4:      	slli	a0, a0, 0xe
   119b6:      	or	a1, a0, s1
   119ba:      	ld	a0, 0x1b0(sp)
   119bc:      	ld	s1, 0x1a8(sp)
   119be:      	xor	s1, s1, a0
   119c0:      	ld	a2, 0x1a0(sp)
   119c2:      	and	s1, s1, a2
   119c4:      	xor	s1, s1, a0
   119c6:      	ld	a0, 0x118(sp)
   119c8:      	add	s1, s1, a0
   119ca:      	ld	a0, 0x138(sp)
   119cc:      	add	s1, s1, a0
   119ce:      	ld	a0, 0x130(sp)
   119d0:      	add	a0, a0, t3
   119d2:      	sd	a0, 0x150(sp)
   119d4:      	ld	a0, 0x128(sp)
   119d6:      	ld	a2, 0xf0(sp)
   119d8:      	xor	a0, a0, a2
   119da:      	ld	a2, 0xe8(sp)
   119dc:      	ld	a3, 0xe0(sp)
   119de:      	xor	a2, a2, a3
   119e0:      	ld	a3, 0xd8(sp)
   119e2:      	ld	a4, 0xd0(sp)
   119e4:      	xor	a3, a3, a4
   119e6:      	ld	a4, 0xc8(sp)
   119e8:      	ld	a5, 0xb8(sp)
   119ea:      	xor	a4, a4, a5
   119ec:      	ld	a5, 0xb0(sp)
   119ee:      	xor	s4, t1, a5
   119f2:      	ld	s0, 0xa8(sp)
   119f4:      	ld	a5, 0xa0(sp)
   119f6:      	xor	s6, s0, a5
   119fa:      	ld	a5, 0x88(sp)
   119fc:      	ld	s0, 0x80(sp)
   119fe:      	xor	t1, a5, s0
   11a02:      	ld	a5, 0x78(sp)
   11a04:      	ld	s0, 0x70(sp)
   11a06:      	xor	t3, a5, s0
   11a0a:      	ld	a5, 0x68(sp)
   11a0c:      	xor	s9, a5, s9
   11a10:      	ld	a5, 0x60(sp)
   11a12:      	xor	s7, a5, s7
   11a16:      	xor	s5, t4, s5
   11a1a:      	xor	t4, s10, t2
   11a1e:      	xor	t5, a7, t5
   11a22:      	xor	s10, s8, s11
   11a26:      	xor	a5, ra, t0
   11a2a:      	sd	a5, 0x138(sp)
   11a2c:      	xor	a5, s2, a6
   11a30:      	sd	a5, 0x180(sp)
   11a32:      	xor	a1, s3, a1
   11a36:      	sd	a1, 0x190(sp)
   11a38:      	lui	a1, 0x428a3
   11a3c:      	addi	a1, a1, -0x68
   11a40:      	ld	t0, 0x198(sp)
   11a42:      	add	a1, a1, t0
   11a44:      	add	s11, s1, a1
   11a48:      	ld	s0, 0x170(sp)
   11a4a:      	srliw	s1, s0, 0x3
   11a4e:      	xor	a0, a0, s1
   11a50:      	sd	a0, 0xd8(sp)
   11a52:      	ld	a0, 0x120(sp)
   11a54:      	srliw	a0, a0, 0x3
   11a58:      	xor	a0, a0, a2
   11a5a:      	sd	a0, 0xf0(sp)
   11a5c:      	ld	a7, 0x38(sp)
   11a5e:      	srliw	a0, a7, 0x3
   11a62:      	xor	s3, a3, a0
   11a66:      	srliw	a2, t6, 0xa
   11a6a:      	xor	a2, a2, a4
   11a6c:      	sd	a2, 0x188(sp)
   11a6e:      	ld	ra, 0x140(sp)
   11a70:      	srliw	a2, ra, 0x3
   11a74:      	xor	s4, s4, a2
   11a78:      	ld	a6, 0x158(sp)
   11a7a:      	srliw	a3, a6, 0x3
   11a7e:      	xor	s8, s6, a3
   11a82:      	ld	s6, 0x98(sp)
   11a84:      	srliw	a5, s6, 0x3
   11a88:      	xor	t2, t1, a5
   11a8c:      	srliw	s1, s0, 0xa
   11a90:      	mv	a5, s0
   11a92:      	xor	s2, t3, s1
   11a96:      	ld	a1, 0x148(sp)
   11a98:      	srliw	a4, a1, 0x3
   11a9c:      	xor	a0, s9, a4
   11aa0:      	sd	a0, 0x130(sp)
   11aa2:      	ld	s9, 0x178(sp)
   11aa4:      	srliw	a4, s9, 0x3
   11aa8:      	xor	a0, s7, a4
   11aac:      	sd	a0, 0x128(sp)
   11aae:      	ld	a0, 0x1c0(sp)
   11ab0:      	srliw	a4, a0, 0x3
   11ab4:      	xor	a2, s5, a4
   11ab8:      	sd	a2, 0xe8(sp)
   11aba:      	ld	a2, 0x1b8(sp)
   11abc:      	srliw	a4, a2, 0x3
   11ac0:      	xor	s5, t4, a4
   11ac4:      	ld	a2, 0x168(sp)
   11ac6:      	srliw	s0, a2, 0x3
   11aca:      	xor	t1, t5, s0
   11ace:      	ld	t3, 0x160(sp)
   11ad0:      	srliw	s0, t3, 0x3
   11ad4:      	xor	s0, s10, s0
   11ad8:      	sd	t6, 0xc0(sp)
   11ada:      	srliw	s1, t6, 0x3
   11ade:      	ld	a3, 0x138(sp)
   11ae0:      	xor	a3, a3, s1
   11ae2:      	sd	a3, 0xc8(sp)
   11ae4:      	ld	s10, 0x1d0(sp)
   11ae6:      	srliw	s1, s10, 0x3
   11aea:      	ld	a3, 0x180(sp)
   11aec:      	xor	a3, a3, s1
   11aee:      	sd	a3, 0xa8(sp)
   11af0:      	ld	a4, 0x1c8(sp)
   11af2:      	srliw	s1, a4, 0x3
   11af6:      	ld	a3, 0x190(sp)
   11af8:      	xor	a3, a3, s1
   11afa:      	sd	a3, 0xb0(sp)
   11afc:      	add	s1, t0, a0
   11b00:      	add	s1, s1, s3
   11b02:      	add	a0, a2, t6
   11b06:      	add	a0, a0, s4
   11b08:      	sd	a0, 0x190(sp)
   11b0a:      	add	a3, t3, s10
   11b0e:      	add	s8, s8, a3
   11b10:      	add	a0, a7, a4
   11b14:      	add	a0, a0, t2
   11b16:      	add	t5, ra, a5
   11b1a:      	add	t5, t5, s5
   11b1c:      	sd	t5, 0xa0(sp)
   11b1e:      	add	a4, a6, a1
   11b22:      	add	a4, a4, t1
   11b24:      	sd	a4, 0xb8(sp)
   11b26:      	add	a4, s6, s9
   11b2a:      	add	t4, s0, a4
   11b2e:      	ld	t1, 0x188(sp)
   11b30:      	add	t1, t1, s1
   11b32:      	add	a6, a0, s2
   11b36:      	ld	a2, 0x150(sp)
   11b38:      	add	a2, a2, s11
   11b3a:      	ld	a0, 0x110(sp)
   11b3c:      	add	a1, s11, a0
   11b40:      	srliw	a0, a1, 0x6
   11b44:      	slli	a4, a1, 0x1a
   11b48:      	or	a0, a0, a4
   11b4a:      	srliw	a4, a1, 0xb
   11b4e:      	slli	a5, a1, 0x15
   11b52:      	or	a4, a4, a5
   11b54:      	srliw	a5, a1, 0x19
   11b58:      	slli	s1, a1, 0x7
   11b5c:      	or	t0, s1, a5
   11b60:      	mv	s1, a2
   11b62:      	srliw	a5, a2, 0x2
   11b66:      	slli	a2, a2, 0x1e
   11b68:      	or	a2, a2, a5
   11b6a:      	srliw	a5, s1, 0xd
   11b6e:      	slli	a3, s1, 0x13
   11b72:      	or	a3, a3, a5
   11b74:      	srliw	a5, s1, 0x16
   11b78:      	slli	s0, s1, 0xa
   11b7c:      	or	s3, s0, a5
   11b80:      	ld	t6, 0x100(sp)
   11b82:      	ld	t2, 0xf8(sp)
   11b84:      	xor	s0, t6, t2
   11b88:      	and	s0, s0, s1
   11b8a:      	mv	s9, s1
   11b8c:      	and	s1, t6, t2
   11b90:      	xor	s2, s0, s1
   11b94:      	ld	a5, 0x1b0(sp)
   11b96:      	add	s11, a5, a7
   11b9a:      	ld	a7, 0x1a8(sp)
   11b9c:      	ld	t5, 0x1a0(sp)
   11b9e:      	xor	s1, a7, t5
   11ba2:      	and	s1, s1, a1
   11ba4:      	xor	s1, s1, a7
   11ba8:      	add	s11, s11, s1
   11baa:      	xor	s7, a0, a4
   11bae:      	xor	s4, a2, a3
   11bb2:      	srliw	a3, a6, 0x11
   11bb6:      	slli	a4, a6, 0xf
   11bba:      	or	s5, a4, a3
   11bbe:      	srliw	a4, a6, 0x13
   11bc2:      	slli	s0, a6, 0xd
   11bc6:      	or	s10, s0, a4
   11bca:      	srliw	s0, t1, 0x11
   11bce:      	slli	a5, t1, 0xf
   11bd2:      	or	ra, a5, s0
   11bd6:      	srliw	s0, t1, 0x13
   11bda:      	slli	s1, t1, 0xd
   11bde:      	or	s0, s0, s1
   11be0:      	srliw	s1, t1, 0x7
   11be4:      	slli	a2, t1, 0x19
   11be8:      	or	a2, a2, s1
   11bea:      	srliw	s1, t1, 0x12
   11bee:      	slli	a3, t1, 0xe
   11bf2:      	sd	t1, 0x180(sp)
   11bf4:      	or	a3, a3, s1
   11bf6:      	srliw	s1, a6, 0x7
   11bfa:      	slli	a0, a6, 0x19
   11bfe:      	or	a0, a0, s1
   11c00:      	srliw	s1, a6, 0x12
   11c04:      	slli	a4, a6, 0xe
   11c08:      	sd	a6, 0x188(sp)
   11c0a:      	or	a4, a4, s1
   11c0c:      	xor	t0, s7, t0
   11c10:      	lui	a5, 0x71374
   11c14:      	addi	a5, a5, 0x491
   11c18:      	add	a5, a5, s11
   11c1a:      	xor	s1, s4, s3
   11c1e:      	xor	s3, s5, s10
   11c22:      	xor	s0, ra, s0
   11c26:      	xor	a2, a2, a3
   11c28:      	xor	a0, a0, a4
   11c2a:      	add	a5, a5, t0
   11c2c:      	add	s1, s1, s2
   11c2e:      	srliw	a3, a6, 0xa
   11c32:      	xor	a3, s3, a3
   11c36:      	srliw	a4, t1, 0xa
   11c3a:      	xor	a4, a4, s0
   11c3c:      	srliw	s0, t1, 0x3
   11c40:      	xor	a2, a2, s0
   11c42:      	sd	a2, 0x60(sp)
   11c44:      	srliw	a2, a6, 0x3
   11c48:      	xor	a0, a0, a2
   11c4a:      	sd	a0, 0xd0(sp)
   11c4c:      	add	a6, s8, a3
   11c50:      	add	t1, t4, a4
   11c54:      	add	s4, s1, a5
   11c58:      	ld	s2, 0x108(sp)
   11c5a:      	add	s2, s2, a5
   11c5c:      	srliw	a0, s2, 0x6
   11c60:      	slli	a3, s2, 0x1a
   11c64:      	or	t4, a3, a0
   11c68:      	srliw	a0, s2, 0xb
   11c6c:      	slli	a5, s2, 0x15
   11c70:      	or	s3, a5, a0
   11c74:      	srliw	a0, s2, 0x19
   11c78:      	slli	s1, s2, 0x7
   11c7c:      	or	t3, s1, a0
   11c80:      	srliw	a0, s4, 0x2
   11c84:      	slli	a2, s4, 0x1e
   11c88:      	or	s8, a2, a0
   11c8c:      	srliw	a2, s4, 0xd
   11c90:      	slli	a4, s4, 0x13
   11c94:      	or	s10, a4, a2
   11c98:      	srliw	a4, s4, 0x16
   11c9c:      	slli	a3, s4, 0xa
   11ca0:      	or	s7, a3, a4
   11ca4:      	sd	s9, 0x138(sp)
   11ca6:      	xor	a4, s9, t2
   11caa:      	and	a4, s4, a4
   11cae:      	and	s1, s9, t2
   11cb2:      	mv	s9, t2
   11cb4:      	xor	s5, a4, s1
   11cb8:      	srliw	s1, a6, 0x11
   11cbc:      	slli	a5, a6, 0xf
   11cc0:      	or	s11, a5, s1
   11cc4:      	srliw	s1, a6, 0x13
   11cc8:      	slli	a4, a6, 0xd
   11ccc:      	or	ra, a4, s1
   11cd0:      	srliw	s1, t1, 0x11
   11cd4:      	slli	a3, t1, 0xf
   11cd8:      	or	t2, a3, s1
   11cdc:      	srliw	s1, t1, 0x13
   11ce0:      	slli	a0, t1, 0xd
   11ce4:      	or	a0, a0, s1
   11ce6:      	srliw	s1, a6, 0x7
   11cea:      	slli	a2, a6, 0x19
   11cee:      	or	a2, a2, s1
   11cf0:      	srliw	s1, a6, 0x12
   11cf4:      	slli	a5, a6, 0xe
   11cf8:      	or	a5, a5, s1
   11cfa:      	srliw	s1, t1, 0x7
   11cfe:      	slli	a4, t1, 0x19
   11d02:      	or	a4, a4, s1
   11d04:      	srliw	s1, t1, 0x12
   11d08:      	slli	t0, t1, 0xe
   11d0c:      	or	t0, t0, s1
   11d10:      	add	s1, a7, s6
   11d14:      	xor	a7, a1, t5
   11d18:      	and	a3, s2, a7
   11d1c:      	xor	a3, a3, t5
   11d20:      	add	a7, s1, a3
   11d24:      	xor	s1, t4, s3
   11d28:      	xor	a3, s8, s10
   11d2c:      	xor	t4, s11, ra
   11d30:      	xor	a0, t2, a0
   11d34:      	xor	a2, a2, a5
   11d36:      	xor	a4, a4, t0
   11d3a:      	xor	a5, s1, t3
   11d3e:      	lui	s1, 0xb5c10
   11d42:      	addi	s1, s1, -0x431
   11d46:      	add	a7, a7, s1
   11d48:      	xor	a3, a3, s7
   11d4c:      	sd	a6, 0x198(sp)
   11d4e:      	srliw	s1, a6, 0xa
   11d52:      	xor	s1, t4, s1
   11d56:      	srliw	s0, t1, 0xa
   11d5a:      	xor	a0, a0, s0
   11d5c:      	srliw	s0, a6, 0x3
   11d60:      	xor	a2, a2, s0
   11d62:      	sd	a2, 0x88(sp)
   11d64:      	srliw	a2, t1, 0x3
   11d68:      	sd	t1, 0xe0(sp)
   11d6a:      	xor	a2, a2, a4
   11d6c:      	sd	a2, 0x78(sp)
   11d6e:      	add	a7, a7, a5
   11d70:      	add	a3, a3, s5
   11d72:      	ld	s6, 0x190(sp)
   11d74:      	add	s6, s6, s1
   11d76:      	ld	t2, 0xb8(sp)
   11d78:      	add	t4, t2, a0
   11d7c:      	srliw	a0, s6, 0x11
   11d80:      	slli	s0, s6, 0xf
   11d84:      	or	a6, s0, a0
   11d88:      	srliw	a0, s6, 0x13
   11d8c:      	slli	a2, s6, 0xd
   11d90:      	or	s3, a2, a0
   11d94:      	srliw	a0, t4, 0x11
   11d98:      	slli	a2, t4, 0xf
   11d9c:      	or	s5, a2, a0
   11da0:      	srliw	a0, t4, 0x13
   11da4:      	slli	a2, t4, 0xd
   11da8:      	or	t0, a2, a0
   11dac:      	srliw	a0, s6, 0x7
   11db0:      	slli	s1, s6, 0x19
   11db4:      	or	t2, s1, a0
   11db8:      	srliw	a0, s6, 0x12
   11dbc:      	slli	a5, s6, 0xe
   11dc0:      	or	s7, a5, a0
   11dc4:      	srliw	a0, t4, 0x7
   11dc8:      	slli	a4, t4, 0x19
   11dcc:      	or	s8, a4, a0
   11dd0:      	srliw	a0, t4, 0x12
   11dd4:      	slli	s0, t4, 0xe
   11dd8:      	or	s11, s0, a0
   11ddc:      	ld	a0, 0x158(sp)
   11dde:      	add	a0, a0, a1
   11de0:      	sd	a0, 0x158(sp)
   11de2:      	add	s10, a3, a7
   11de6:      	add	a7, a7, t6
   11de8:      	xor	a2, s2, a1
   11dec:      	and	a2, a7, a2
   11df0:      	xor	t3, a2, a1
   11df4:      	srliw	a2, a7, 0x6
   11df8:      	slli	a0, a7, 0x1a
   11dfc:      	or	a0, a0, a2
   11dfe:      	srliw	a2, a7, 0xb
   11e02:      	slli	s1, a7, 0x15
   11e06:      	or	a2, a2, s1
   11e08:      	srliw	s1, a7, 0x19
   11e0c:      	slli	a5, a7, 0x7
   11e10:      	or	ra, a5, s1
   11e14:      	srliw	s1, s10, 0x2
   11e18:      	slli	a3, s10, 0x1e
   11e1c:      	or	a3, a3, s1
   11e1e:      	srliw	s1, s10, 0xd
   11e22:      	slli	a4, s10, 0x13
   11e26:      	or	a4, a4, s1
   11e28:      	srliw	s1, s10, 0x16
   11e2c:      	slli	s0, s10, 0xa
   11e30:      	or	s0, s0, s1
   11e32:      	ld	a1, 0x138(sp)
   11e34:      	xor	s1, s4, a1
   11e38:      	and	s1, s10, s1
   11e3c:      	and	a5, s4, a1
   11e40:      	xor	a5, a5, s1
   11e42:      	xor	s1, a6, s3
   11e46:      	xor	a6, s5, t0
   11e4a:      	xor	t0, t2, s7
   11e4e:      	xor	t2, s8, s11
   11e52:      	ld	a1, 0x160(sp)
   11e54:      	add	a1, a1, t5
   11e56:      	add	a1, a1, t3
   11e58:      	xor	a0, a0, a2
   11e5a:      	xor	a3, a3, a4
   11e5c:      	sd	s6, 0x150(sp)
   11e5e:      	srliw	a2, s6, 0xa
   11e62:      	xor	a2, a2, s1
   11e64:      	sd	t4, 0x190(sp)
   11e66:      	srliw	a4, t4, 0xa
   11e6a:      	xor	a4, a6, a4
   11e6e:      	srliw	s1, s6, 0x3
   11e72:      	xor	s1, t0, s1
   11e76:      	sd	s1, 0xb8(sp)
   11e78:      	srliw	s1, t4, 0x3
   11e7c:      	xor	s1, t2, s1
   11e80:      	sd	s1, 0x70(sp)
   11e82:      	xor	a0, a0, ra
   11e86:      	lui	s1, 0xe9b5e
   11e8a:      	addi	s1, s1, -0x45b
   11e8e:      	add	s1, s1, a1
   11e90:      	xor	a3, a3, s0
   11e92:      	ld	s6, 0x1b8(sp)
   11e94:      	ld	a1, 0xf0(sp)
   11e96:      	add	s6, s6, a1
   11e98:      	ld	a1, 0x180(sp)
   11e9a:      	add	s6, s6, a1
   11e9c:      	add	t6, s6, a2
   11ea0:      	ld	a1, 0xa0(sp)
   11ea2:      	add	ra, a1, a4
   11ea6:      	add	s7, s1, a0
   11eaa:      	add	a6, a3, a5
   11eae:      	srliw	a2, ra, 0x11
   11eb2:      	slli	a3, ra, 0xf
   11eb6:      	or	t0, a3, a2
   11eba:      	srliw	a3, ra, 0x13
   11ebe:      	slli	a4, ra, 0xd
   11ec2:      	or	t2, a4, a3
   11ec6:      	srliw	a4, t6, 0x11
   11eca:      	slli	a5, t6, 0xf
   11ece:      	or	a4, a4, a5
   11ed0:      	srliw	a5, t6, 0x13
   11ed4:      	slli	s1, t6, 0xd
   11ed8:      	or	a5, a5, s1
   11eda:      	srliw	s1, t6, 0x7
   11ede:      	slli	s0, t6, 0x19
   11ee2:      	or	s0, s0, s1
   11ee4:      	srliw	s1, t6, 0x12
   11ee8:      	slli	a0, t6, 0xe
   11eec:      	or	a0, a0, s1
   11eee:      	srliw	s1, ra, 0x7
   11ef2:      	slli	a2, ra, 0x19
   11ef6:      	or	a2, a2, s1
   11ef8:      	srliw	s1, ra, 0x12
   11efc:      	slli	a3, ra, 0xe
   11f00:      	or	a3, a3, s1
   11f02:      	xor	t3, t0, t2
   11f06:      	xor	t2, a4, a5
   11f0a:      	xor	s6, s0, a0
   11f0e:      	xor	s3, a2, a3
   11f12:      	ld	t0, 0x168(sp)
   11f14:      	add	t0, t0, s2
   11f16:      	add	t5, a6, s7
   11f1a:      	add	a0, s7, s9
   11f1e:      	xor	a3, a7, s2
   11f22:      	and	a3, a3, a0
   11f24:      	xor	a6, a3, s2
   11f28:      	srliw	a5, a0, 0x6
   11f2c:      	slli	a1, a0, 0x1a
   11f30:      	or	s7, a1, a5
   11f34:      	srliw	a5, a0, 0xb
   11f38:      	slli	a4, a0, 0x15
   11f3c:      	or	s8, a4, a5
   11f40:      	srliw	a5, a0, 0x19
   11f44:      	slli	s1, a0, 0x7
   11f48:      	or	s2, s1, a5
   11f4c:      	srliw	s1, t5, 0x2
   11f50:      	slli	a2, t5, 0x1e
   11f54:      	or	a2, a2, s1
   11f56:      	srliw	s1, t5, 0xd
   11f5a:      	slli	s0, t5, 0x13
   11f5e:      	or	s0, s0, s1
   11f60:      	srliw	s1, t5, 0x16
   11f64:      	slli	a3, t5, 0xa
   11f68:      	or	a3, a3, s1
   11f6a:      	xor	s1, s10, s4
   11f6e:      	and	s1, t5, s1
   11f72:      	and	a5, s10, s4
   11f76:      	xor	s11, s1, a5
   11f7a:      	srliw	s1, ra, 0xa
   11f7e:      	xor	s1, t3, s1
   11f82:      	sd	t6, 0x160(sp)
   11f84:      	srliw	a1, t6, 0xa
   11f88:      	xor	a1, t2, a1
   11f8c:      	srliw	a4, t6, 0x3
   11f90:      	xor	a4, s6, a4
   11f94:      	sd	a4, 0x98(sp)
   11f96:      	srliw	a4, ra, 0x3
   11f9a:      	mv	s6, ra
   11f9c:      	sd	ra, 0xf0(sp)
   11f9e:      	xor	a4, s3, a4
   11fa2:      	sd	a4, 0x80(sp)
   11fa4:      	ld	a4, 0x158(sp)
   11fa6:      	add	a6, a6, a4
   11fa8:      	xor	a4, s7, s8
   11fac:      	xor	a2, a2, s0
   11fae:      	ld	s0, 0x120(sp)
   11fb0:      	ld	a5, 0xe8(sp)
   11fb2:      	add	s0, s0, a5
   11fb4:      	ld	a5, 0x188(sp)
   11fb6:      	add	s0, s0, a5
   11fb8:      	add	s5, s0, s1
   11fbc:      	ld	s9, 0x1c0(sp)
   11fbe:      	ld	s1, 0xb0(sp)
   11fc0:      	add	s9, s9, s1
   11fc2:      	add	s9, s9, t1
   11fc4:      	add	s9, s9, a1
   11fc6:      	xor	a1, a4, s2
   11fca:      	lui	a4, 0x3956c
   11fce:      	addi	a4, a4, 0x25b
   11fd2:      	add	a4, a4, a6
   11fd4:      	xor	a2, a2, a3
   11fd6:      	add	a1, a1, a4
   11fd8:      	add	s11, s11, a2
   11fda:      	srliw	a2, s5, 0x11
   11fde:      	slli	a3, s5, 0xf
   11fe2:      	or	a6, a3, a2
   11fe6:      	srliw	a2, s5, 0x13
   11fea:      	slli	a4, s5, 0xd
   11fee:      	or	t2, a4, a2
   11ff2:      	srliw	a2, s9, 0x11
   11ff6:      	slli	s1, s9, 0xf
   11ffa:      	or	t3, s1, a2
   11ffe:      	srliw	a2, s9, 0x13
   12002:      	slli	s0, s9, 0xd
   12006:      	or	s0, s0, a2
   12008:      	srliw	a2, s5, 0x7
   1200c:      	slli	a5, s5, 0x19
   12010:      	or	t4, a5, a2
   12014:      	srliw	a2, s5, 0x12
   12018:      	slli	a3, s5, 0xe
   1201c:      	or	a3, a3, a2
   1201e:      	srliw	a2, s9, 0x7
   12022:      	slli	a4, s9, 0x19
   12026:      	or	a4, a4, a2
   12028:      	srliw	a2, s9, 0x12
   1202c:      	slli	s1, s9, 0xe
   12030:      	or	s1, s1, a2
   12032:      	ld	t1, 0x138(sp)
   12034:      	add	t1, t1, a1
   12036:      	add	s11, s11, a1
   12038:      	xor	a1, a6, t2
   1203c:      	xor	s0, t3, s0
   12040:      	xor	a2, t4, a3
   12044:      	xor	a4, a4, s1
   12046:      	srliw	a3, s5, 0xa
   1204a:      	xor	t3, a1, a3
   1204e:      	sd	s9, 0x158(sp)
   12050:      	srliw	a1, s9, 0xa
   12054:      	xor	t2, s0, a1
   12058:      	srliw	s1, s5, 0x3
   1205c:      	sd	s5, 0xe8(sp)
   1205e:      	xor	a2, a2, s1
   12060:      	sd	a2, 0x68(sp)
   12062:      	srliw	a5, s9, 0x3
   12066:      	xor	a4, a4, a5
   12068:      	sd	a4, 0xb0(sp)
   1206a:      	ld	ra, 0x140(sp)
   1206c:      	add	ra, ra, a7
   1206e:      	xor	a4, a0, a7
   12072:      	and	a4, t1, a4
   12076:      	xor	a7, a4, a7
   1207a:      	srliw	a5, t1, 0x6
   1207e:      	slli	a2, t1, 0x1a
   12082:      	or	t6, a2, a5
   12086:      	srliw	a5, t1, 0xb
   1208a:      	slli	s1, t1, 0x15
   1208e:      	or	a5, a5, s1
   12090:      	srliw	s1, t1, 0x19
   12094:      	slli	s0, t1, 0x7
   12098:      	or	s0, s0, s1
   1209a:      	srliw	s1, s11, 0x2
   1209e:      	slli	a1, s11, 0x1e
   120a2:      	or	a1, a1, s1
   120a4:      	srliw	s1, s11, 0xd
   120a8:      	slli	a3, s11, 0x13
   120ac:      	or	a3, a3, s1
   120ae:      	srliw	s1, s11, 0x16
   120b2:      	slli	a4, s11, 0xa
   120b6:      	or	a4, a4, s1
   120b8:      	xor	s1, t5, s10
   120bc:      	and	s1, s11, s1
   120c0:      	and	a2, t5, s10
   120c4:      	xor	s2, s1, a2
   120c8:      	ld	s1, 0x1c8(sp)
   120ca:      	ld	a2, 0x128(sp)
   120cc:      	add	s1, s1, a2
   120ce:      	ld	a2, 0x198(sp)
   120d0:      	add	s1, s1, a2
   120d2:      	add	t3, t3, s1
   120d4:      	ld	s1, 0x178(sp)
   120d6:      	ld	a2, 0xa8(sp)
   120d8:      	add	s1, s1, a2
   120da:      	ld	a2, 0x190(sp)
   120dc:      	add	s1, s1, a2
   120de:      	add	a2, s1, t2
   120e2:      	add	a7, a7, t0
   120e4:      	xor	a5, t6, a5
   120e8:      	xor	a1, a1, a3
   120ea:      	xor	t0, a5, s0
   120ee:      	lui	a3, 0x59f11
   120f2:      	addi	a3, a3, 0x1f1
   120f6:      	add	t2, a7, a3
   120fa:      	xor	a7, a1, a4
   120fe:      	mv	a3, t3
   12100:      	srliw	a4, t3, 0x11
   12104:      	slli	s1, t3, 0xf
   12108:      	or	t3, s1, a4
   1210c:      	srliw	s1, a3, 0x13
   12110:      	slli	s0, a3, 0xd
   12114:      	or	t6, s0, s1
   12118:      	mv	s0, a2
   1211a:      	srliw	s1, a2, 0x11
   1211e:      	slli	a2, a2, 0xf
   12120:      	or	a6, a2, s1
   12124:      	srliw	s1, s0, 0x13
   12128:      	slli	a5, s0, 0xd
   1212c:      	or	a2, a5, s1
   12130:      	srliw	s1, a3, 0x7
   12134:      	slli	a1, a3, 0x19
   12138:      	or	t4, a1, s1
   1213c:      	srliw	s1, a3, 0x12
   12140:      	mv	a1, a3
   12142:      	slli	a3, a3, 0xe
   12144:      	or	a3, a3, s1
   12146:      	srliw	s1, s0, 0x7
   1214a:      	slli	a4, s0, 0x19
   1214e:      	or	a4, a4, s1
   12150:      	srliw	s1, s0, 0x12
   12154:      	mv	a5, s0
   12156:      	slli	s0, s0, 0xe
   12158:      	or	s0, s0, s1
   1215a:      	add	t0, t0, t2
   1215c:      	add	a7, a7, s2
   1215e:      	xor	s1, t3, t6
   12162:      	xor	a2, a6, a2
   12166:      	xor	a6, t4, a3
   1216a:      	xor	a4, a4, s0
   1216c:      	add	s4, s4, t0
   1216e:      	add	t3, a7, t0
   12172:      	sd	a1, 0x168(sp)
   12174:      	srliw	a3, a1, 0xa
   12178:      	xor	a3, a3, s1
   1217a:      	mv	s1, a5
   1217c:      	sd	a5, 0x128(sp)
   1217e:      	srliw	a5, a5, 0xa
   12182:      	xor	a2, a2, a5
   12184:      	srliw	a5, a1, 0x3
   12188:      	xor	a1, a6, a5
   1218c:      	sd	a1, 0x30(sp)
   1218e:      	srliw	a1, s1, 0x3
   12192:      	xor	a1, a1, a4
   12194:      	sd	a1, 0xa8(sp)
   12196:      	ld	a1, 0x1d0(sp)
   12198:      	ld	a4, 0x130(sp)
   1219a:      	add	a1, a1, a4
   1219c:      	ld	a4, 0x150(sp)
   1219e:      	add	a1, a1, a4
   121a0:      	add	t4, a1, a3
   121a4:      	ld	a1, 0x148(sp)
   121a6:      	ld	a3, 0xc8(sp)
   121a8:      	add	a1, a1, a3
   121aa:      	add	a1, a1, s6
   121ac:      	add	a6, a1, a2
   121b0:      	ld	t2, 0x1b8(sp)
   121b2:      	add	t2, t2, a0
   121b4:      	xor	a1, t1, a0
   121b8:      	and	a1, s4, a1
   121bc:      	xor	t6, a1, a0
   121c0:      	srliw	a1, s4, 0x6
   121c4:      	slli	a2, s4, 0x1a
   121c8:      	or	a2, a2, a1
   121ca:      	srliw	a1, s4, 0xb
   121ce:      	slli	a3, s4, 0x15
   121d2:      	or	a3, a3, a1
   121d4:      	srliw	a1, s4, 0x19
   121d8:      	slli	a4, s4, 0x7
   121dc:      	or	t0, a4, a1
   121e0:      	srliw	a4, t3, 0x2
   121e4:      	slli	s1, t3, 0x1e
   121e8:      	or	a4, a4, s1
   121ea:      	srliw	s1, t3, 0xd
   121ee:      	slli	a5, t3, 0x13
   121f2:      	or	a5, a5, s1
   121f4:      	srliw	s1, t3, 0x16
   121f8:      	slli	a1, t3, 0xa
   121fc:      	or	s3, a1, s1
   12200:      	xor	s1, s11, t5
   12204:      	and	s1, t3, s1
   12208:      	and	a0, s11, t5
   1220c:      	xor	s2, s1, a0
   12210:      	add	t6, t6, ra
   12212:      	xor	s7, a2, a3
   12216:      	xor	s0, a4, a5
   1221a:      	srliw	a3, t4, 0x11
   1221e:      	slli	a5, t4, 0xf
   12222:      	or	s6, a5, a3
   12226:      	srliw	a5, t4, 0x13
   1222a:      	slli	s1, t4, 0xd
   1222e:      	or	s8, s1, a5
   12232:      	srliw	s1, a6, 0x11
   12236:      	slli	a0, a6, 0xf
   1223a:      	or	s9, a0, s1
   1223e:      	srliw	s1, a6, 0x13
   12242:      	slli	a1, a6, 0xd
   12246:      	or	ra, a1, s1
   1224a:      	srliw	s1, t4, 0x7
   1224e:      	slli	a4, t4, 0x19
   12252:      	or	a4, a4, s1
   12254:      	srliw	s1, t4, 0x12
   12258:      	slli	a3, t4, 0xe
   1225c:      	sd	t4, 0x140(sp)
   1225e:      	or	a3, a3, s1
   12260:      	srliw	s1, a6, 0x7
   12264:      	slli	a2, a6, 0x19
   12268:      	or	a2, a2, s1
   1226a:      	srliw	s1, a6, 0x12
   1226e:      	slli	a5, a6, 0xe
   12272:      	sd	a6, 0x130(sp)
   12274:      	or	a5, a5, s1
   12276:      	xor	t0, s7, t0
   1227a:      	lui	a0, 0x923f8
   1227e:      	addi	a0, a0, 0x2a4
   12282:      	add	a0, a0, t6
   12284:      	xor	a1, s0, s3
   12288:      	xor	s1, s6, s8
   1228c:      	xor	t6, s9, ra
   12290:      	xor	a3, a3, a4
   12292:      	xor	a2, a2, a5
   12294:      	add	a0, a0, t0
   12296:      	add	a1, a1, s2
   12298:      	srliw	a4, t4, 0xa
   1229c:      	xor	a4, a4, s1
   1229e:      	srliw	a5, a6, 0xa
   122a2:      	xor	a5, t6, a5
   122a6:      	srliw	s1, t4, 0x3
   122aa:      	xor	a3, a3, s1
   122ac:      	sd	a3, 0xa0(sp)
   122ae:      	srliw	a3, a6, 0x3
   122b2:      	xor	a2, a2, a3
   122b4:      	sd	a2, 0x50(sp)
   122b6:      	add	s10, s10, a0
   122b8:      	add	t6, a1, a0
   122bc:      	ld	a0, 0xc0(sp)
   122be:      	ld	a1, 0xd8(sp)
   122c0:      	add	a0, a0, a1
   122c2:      	ld	a1, 0x160(sp)
   122c4:      	add	a0, a0, a1
   122c6:      	add	a7, a0, a4
   122ca:      	ld	a0, 0x170(sp)
   122cc:      	ld	a1, 0x60(sp)
   122ce:      	add	a0, a0, a1
   122d0:      	add	a0, a0, s5
   122d2:      	add	a3, a0, a5
   122d6:      	ld	a0, 0x120(sp)
   122d8:      	add	a0, a0, t1
   122da:      	sd	a0, 0x1b8(sp)
   122dc:      	xor	a0, s4, t1
   122e0:      	and	a0, s10, a0
   122e4:      	xor	t1, a0, t1
   122e8:      	srliw	a0, s10, 0x6
   122ec:      	slli	a1, s10, 0x1a
   122f0:      	or	s2, a1, a0
   122f4:      	srliw	a0, s10, 0xb
   122f8:      	slli	s1, s10, 0x15
   122fc:      	or	t4, s1, a0
   12300:      	srliw	a0, s10, 0x19
   12304:      	slli	a2, s10, 0x7
   12308:      	or	s3, a2, a0
   1230c:      	srliw	a0, t6, 0x2
   12310:      	slli	a5, t6, 0x1e
   12314:      	or	s8, a5, a0
   12318:      	srliw	a5, t6, 0xd
   1231c:      	slli	a4, t6, 0x13
   12320:      	or	s9, a4, a5
   12324:      	srliw	a5, t6, 0x16
   12328:      	slli	a1, t6, 0xa
   1232c:      	or	s7, a1, a5
   12330:      	xor	a5, t3, s11
   12334:      	and	a5, t6, a5
   12338:      	and	a2, t3, s11
   1233c:      	xor	s6, a5, a2
   12340:      	srliw	a5, a7, 0x7
   12344:      	slli	s1, a7, 0x19
   12348:      	or	s5, s1, a5
   1234c:      	srliw	s1, a7, 0x12
   12350:      	slli	a2, a7, 0xe
   12354:      	or	ra, a2, s1
   12358:      	srliw	s1, a7, 0x11
   1235c:      	slli	a1, a7, 0xf
   12360:      	or	a1, a1, s1
   12362:      	srliw	s1, a7, 0x13
   12366:      	slli	a0, a7, 0xd
   1236a:      	or	a0, a0, s1
   1236c:      	srliw	s1, a3, 0x11
   12370:      	slli	a4, a3, 0xf
   12374:      	or	a4, a4, s1
   12376:      	srliw	s1, a3, 0x13
   1237a:      	slli	a5, a3, 0xd
   1237e:      	or	a5, a5, s1
   12380:      	srliw	s1, a3, 0x7
   12384:      	slli	a2, a3, 0x19
   12388:      	or	a2, a2, s1
   1238a:      	srliw	s1, a3, 0x12
   1238e:      	slli	a6, a3, 0xe
   12392:      	or	a6, a6, s1
   12396:      	add	t1, t1, t2
   12398:      	xor	s1, s2, t4
   1239c:      	xor	t2, s8, s9
   123a0:      	xor	s2, s5, ra
   123a4:      	xor	a0, a0, a1
   123a6:      	xor	a4, a4, a5
   123a8:      	xor	a1, a2, a6
   123ac:      	xor	a2, s1, s3
   123b0:      	lui	a5, 0xab1c6
   123b4:      	addi	a5, a5, -0x12b
   123b8:      	add	a5, a5, t1
   123ba:      	xor	s1, t2, s7
   123be:      	srliw	s0, a7, 0x3
   123c2:      	xor	s0, s2, s0
   123c6:      	sd	s0, 0x40(sp)
   123c8:      	srliw	s0, a7, 0xa
   123cc:      	mv	s2, a7
   123ce:      	sd	a7, 0xd8(sp)
   123d0:      	xor	a0, a0, s0
   123d2:      	sd	a3, 0x138(sp)
   123d4:      	srliw	s0, a3, 0xa
   123d8:      	xor	a4, a4, s0
   123da:      	srliw	s0, a3, 0x3
   123de:      	xor	a1, a1, s0
   123e0:      	sd	a1, 0x48(sp)
   123e2:      	add	a2, a2, a5
   123e4:      	add	s1, s1, s6
   123e6:      	ld	a1, 0x180(sp)
   123e8:      	ld	a3, 0xd0(sp)
   123ea:      	add	a1, a1, a3
   123ec:      	ld	a3, 0x158(sp)
   123ee:      	add	a1, a1, a3
   123f0:      	add	a7, a1, a0
   123f4:      	ld	t4, 0x188(sp)
   123f6:      	ld	a0, 0x78(sp)
   123f8:      	add	t4, t4, a0
   123fa:      	ld	a0, 0x168(sp)
   123fc:      	add	t4, t4, a0
   123fe:      	add	s5, t4, a4
   12402:      	add	s3, a2, t5
   12406:      	add	t1, s1, a2
   1240a:      	srliw	a1, a7, 0x11
   1240e:      	slli	a2, a7, 0xf
   12412:      	or	t2, a2, a1
   12416:      	srliw	a1, a7, 0x13
   1241a:      	slli	a2, a7, 0xd
   1241e:      	or	t0, a2, a1
   12422:      	srliw	a1, s5, 0x11
   12426:      	slli	a4, s5, 0xf
   1242a:      	or	t4, a4, a1
   1242e:      	srliw	a1, s5, 0x13
   12432:      	slli	s1, s5, 0xd
   12436:      	or	t5, s1, a1
   1243a:      	srliw	a1, a7, 0x7
   1243e:      	slli	a5, a7, 0x19
   12442:      	or	s6, a5, a1
   12446:      	srliw	s0, a7, 0x12
   1244a:      	slli	a1, a7, 0xe
   1244e:      	or	s7, a1, s0
   12452:      	srliw	s0, s5, 0x7
   12456:      	slli	a2, s5, 0x19
   1245a:      	or	s8, a2, s0
   1245e:      	srliw	s0, s5, 0x12
   12462:      	slli	a4, s5, 0xe
   12466:      	or	s9, a4, s0
   1246a:      	ld	a0, 0x1c0(sp)
   1246c:      	add	a0, a0, s4
   1246e:      	sd	a0, 0x60(sp)
   12470:      	xor	s0, s10, s4
   12474:      	and	s0, s3, s0
   12478:      	xor	a3, s0, s4
   1247c:      	srliw	s1, s3, 0x6
   12480:      	slli	a0, s3, 0x1a
   12484:      	or	ra, a0, s1
   12488:      	srliw	s1, s3, 0xb
   1248c:      	slli	a5, s3, 0x15
   12490:      	or	a5, a5, s1
   12492:      	srliw	s1, s3, 0x19
   12496:      	slli	a1, s3, 0x7
   1249a:      	or	s4, a1, s1
   1249e:      	srliw	s1, t1, 0x2
   124a2:      	slli	a2, t1, 0x1e
   124a6:      	or	a2, a2, s1
   124a8:      	srliw	s1, t1, 0xd
   124ac:      	slli	a4, t1, 0x13
   124b0:      	or	a4, a4, s1
   124b2:      	srliw	s1, t1, 0x16
   124b6:      	slli	a1, t1, 0xa
   124ba:      	or	a1, a1, s1
   124bc:      	xor	s1, t6, t3
   124c0:      	and	s1, t1, s1
   124c4:      	and	s0, t6, t3
   124c8:      	xor	s0, s0, s1
   124ca:      	xor	s1, t2, t0
   124ce:      	xor	a0, t4, t5
   124d2:      	xor	a6, s6, s7
   124d6:      	xor	t2, s8, s9
   124da:      	ld	t0, 0x1b8(sp)
   124dc:      	add	t0, t0, a3
   124de:      	xor	a5, ra, a5
   124e2:      	xor	a2, a2, a4
   124e4:      	mv	t4, a7
   124e6:      	srliw	a4, a7, 0xa
   124ea:      	xor	a4, a4, s1
   124ec:      	mv	s9, s5
   124ee:      	srliw	s1, s5, 0xa
   124f2:      	xor	a0, a0, s1
   124f4:      	srliw	s1, a7, 0x3
   124f8:      	sd	a7, 0xd0(sp)
   124fa:      	xor	a3, a6, s1
   124fe:      	sd	a3, 0x38(sp)
   12500:      	srliw	s1, s5, 0x3
   12504:      	sd	s5, 0xc8(sp)
   12506:      	xor	a3, t2, s1
   1250a:      	sd	a3, 0x78(sp)
   1250c:      	xor	a5, a5, s4
   12510:      	lui	s1, 0xd807b
   12514:      	addi	s1, s1, -0x568
   12518:      	add	t0, t0, s1
   1251a:      	xor	a1, a1, a2
   1251c:      	ld	a2, 0xe0(sp)
   1251e:      	ld	a3, 0x88(sp)
   12520:      	add	a2, a2, a3
   12522:      	ld	a3, 0x128(sp)
   12524:      	add	a2, a2, a3
   12526:      	add	a7, a2, a4
   1252a:      	ld	s5, 0x198(sp)
   1252c:      	ld	a3, 0x70(sp)
   1252e:      	add	s5, s5, a3
   12530:      	ld	a3, 0x140(sp)
   12532:      	add	s5, s5, a3
   12534:      	add	s7, s5, a0
   12538:      	add	a5, a5, t0
   1253a:      	add	a1, a1, s0
   1253c:      	add	s11, s11, a5
   1253e:      	add	ra, a1, a5
   12542:      	srliw	a0, a7, 0x11
   12546:      	slli	a1, a7, 0xf
   1254a:      	or	a6, a1, a0
   1254e:      	srliw	a1, a7, 0x13
   12552:      	slli	a4, a7, 0xd
   12556:      	or	a1, a1, a4
   12558:      	srliw	a4, s7, 0x11
   1255c:      	slli	a5, s7, 0xf
   12560:      	or	a4, a4, a5
   12562:      	srliw	a5, s7, 0x13
   12566:      	slli	s1, s7, 0xd
   1256a:      	or	a5, a5, s1
   1256c:      	srliw	s1, a7, 0x7
   12570:      	slli	s0, a7, 0x19
   12574:      	or	s0, s0, s1
   12576:      	srliw	s1, a7, 0x12
   1257a:      	slli	a0, a7, 0xe
   1257e:      	or	a0, a0, s1
   12580:      	srliw	s1, s7, 0x7
   12584:      	slli	a2, s7, 0x19
   12588:      	or	a2, a2, s1
   1258a:      	srliw	s1, s7, 0x12
   1258e:      	slli	a3, s7, 0xe
   12592:      	or	a3, a3, s1
   12594:      	xor	s1, a6, a1
   12598:      	xor	t5, a4, a5
   1259c:      	xor	t0, s0, a0
   125a0:      	xor	a6, a2, a3
   125a4:      	ld	t2, 0x1c8(sp)
   125a6:      	add	t2, t2, s10
   125a8:      	xor	a0, s3, s10
   125ac:      	and	a0, s11, a0
   125b0:      	xor	s4, a0, s10
   125b4:      	srliw	a0, s11, 0x6
   125b8:      	slli	a2, s11, 0x1a
   125bc:      	or	s6, a2, a0
   125c0:      	srliw	a2, s11, 0xb
   125c4:      	slli	s0, s11, 0x15
   125c8:      	or	a2, a2, s0
   125ca:      	srliw	a1, s11, 0x19
   125ce:      	slli	s0, s11, 0x7
   125d2:      	or	s5, s0, a1
   125d6:      	srliw	a1, ra, 0x2
   125da:      	slli	a4, ra, 0x1e
   125de:      	or	a1, a1, a4
   125e0:      	srliw	a4, ra, 0xd
   125e4:      	slli	a5, ra, 0x13
   125e8:      	or	a4, a4, a5
   125ea:      	srliw	a5, ra, 0x16
   125ee:      	slli	a3, ra, 0xa
   125f2:      	or	a3, a3, a5
   125f4:      	mv	s10, t1
   125f6:      	xor	a5, t1, t6
   125fa:      	and	a5, ra, a5
   125fe:      	and	s0, t1, t6
   12602:      	xor	a5, a5, s0
   12604:      	srliw	s0, a7, 0xa
   12608:      	xor	s0, s0, s1
   1260a:      	sd	s7, 0x1c0(sp)
   1260c:      	srliw	s1, s7, 0xa
   12610:      	xor	s1, t5, s1
   12614:      	srliw	a0, a7, 0x3
   12618:      	mv	t5, a7
   1261a:      	sd	a7, 0x120(sp)
   1261c:      	xor	a0, t0, a0
   12620:      	sd	a0, 0x58(sp)
   12622:      	srliw	a0, s7, 0x3
   12626:      	xor	a0, a6, a0
   1262a:      	sd	a0, 0x88(sp)
   1262c:      	ld	t1, 0x60(sp)
   1262e:      	add	t1, t1, s4
   12630:      	xor	a0, s6, a2
   12634:      	xor	a1, a1, a4
   12636:      	ld	a2, 0x190(sp)
   12638:      	ld	a4, 0xb8(sp)
   1263a:      	add	a2, a2, a4
   1263c:      	ld	a4, 0x130(sp)
   1263e:      	add	a2, a2, a4
   12640:      	add	s0, s0, a2
   12642:      	ld	a4, 0x80(sp)
   12644:      	ld	a2, 0x150(sp)
   12646:      	add	a2, a2, a4
   12648:      	add	a2, a2, s2
   1264a:      	add	s1, s1, a2
   1264c:      	xor	a0, a0, s5
   12650:      	lui	a2, 0x12836
   12654:      	addi	a2, a2, -0x4ff
   12658:      	add	a2, a2, t1
   1265a:      	xor	a1, a1, a3
   1265c:      	add	a0, a0, a2
   1265e:      	add	s8, a1, a5
   12662:      	srliw	a1, s0, 0x11
   12666:      	slli	a2, s0, 0xf
   1266a:      	or	a6, a2, a1
   1266e:      	mv	a2, s0
   12670:      	srliw	a1, s0, 0x13
   12674:      	slli	a3, s0, 0xd
   12678:      	or	t0, a3, a1
   1267c:      	mv	a3, s1
   1267e:      	srliw	a1, s1, 0x11
   12682:      	slli	a4, s1, 0xf
   12686:      	or	a7, a4, a1
   1268a:      	srliw	a1, s1, 0x13
   1268e:      	slli	s1, s1, 0xd
   12690:      	or	t1, s1, a1
   12694:      	srliw	a1, s0, 0x7
   12698:      	slli	s0, s0, 0x19
   1269a:      	or	a4, s0, a1
   1269e:      	srliw	a1, a2, 0x12
   126a2:      	slli	a5, a2, 0xe
   126a6:      	mv	s1, a2
   126a8:      	or	a5, a5, a1
   126aa:      	srliw	a1, a3, 0x7
   126ae:      	slli	a2, a3, 0x19
   126b2:      	or	a2, a2, a1
   126b4:      	srliw	a1, a3, 0x12
   126b8:      	mv	s0, a3
   126ba:      	slli	a3, a3, 0xe
   126bc:      	or	a3, a3, a1
   126be:      	add	t3, t3, a0
   126c0:      	add	s4, s8, a0
   126c4:      	xor	a0, a6, t0
   126c8:      	xor	a1, a7, t1
   126cc:      	xor	a5, a5, a4
   126ce:      	xor	a2, a2, a3
   126d0:      	srliw	a4, s1, 0xa
   126d4:      	xor	t0, a0, a4
   126d8:      	sd	s0, 0x1c8(sp)
   126da:      	srliw	a3, s0, 0xa
   126de:      	xor	a6, a1, a3
   126e2:      	srliw	a0, s1, 0x3
   126e6:      	mv	s5, s1
   126e8:      	sd	s1, 0xb8(sp)
   126ea:      	xor	a0, a0, a5
   126ec:      	sd	a0, 0x70(sp)
   126ee:      	srliw	a0, s0, 0x3
   126f2:      	xor	a0, a0, a2
   126f4:      	sd	a0, 0x80(sp)
   126f6:      	ld	s6, 0x178(sp)
   126f8:      	add	s6, s6, s3
   126fa:      	xor	a0, s11, s3
   126fe:      	and	a0, t3, a0
   12702:      	xor	t1, a0, s3
   12706:      	srliw	a2, t3, 0x6
   1270a:      	slli	a5, t3, 0x1a
   1270e:      	or	a7, a5, a2
   12712:      	srliw	a5, t3, 0xb
   12716:      	slli	a1, t3, 0x15
   1271a:      	or	a1, a1, a5
   1271c:      	srliw	a5, t3, 0x19
   12720:      	slli	s1, t3, 0x7
   12724:      	or	a5, a5, s1
   12726:      	srliw	s1, s4, 0x2
   1272a:      	slli	s0, s4, 0x1e
   1272e:      	or	a2, s0, s1
   12732:      	srliw	s1, s4, 0xd
   12736:      	slli	a3, s4, 0x13
   1273a:      	or	a3, a3, s1
   1273c:      	srliw	s1, s4, 0x16
   12740:      	slli	a4, s4, 0xa
   12744:      	or	a4, a4, s1
   12746:      	xor	s1, ra, s10
   1274a:      	and	s1, s4, s1
   1274e:      	and	a0, ra, s10
   12752:      	xor	s3, s1, a0
   12756:      	ld	s1, 0xf0(sp)
   12758:      	ld	a0, 0x98(sp)
   1275a:      	add	s1, s1, a0
   1275c:      	ld	a0, 0x138(sp)
   1275e:      	add	s1, s1, a0
   12760:      	add	s0, s1, t0
   12764:      	ld	s1, 0x160(sp)
   12766:      	ld	a0, 0x68(sp)
   12768:      	add	s1, s1, a0
   1276a:      	add	s1, s1, t4
   1276c:      	add	a0, s1, a6
   12770:      	add	t1, t1, t2
   12772:      	xor	a1, a7, a1
   12776:      	xor	a3, a3, a2
   12778:      	xor	a6, a1, a5
   1277c:      	lui	a2, 0x24318
   12780:      	addi	a2, a2, 0x5be
   12784:      	add	t1, t1, a2
   12786:      	xor	t0, a3, a4
   1278a:      	srliw	a4, a0, 0x11
   1278e:      	slli	a5, a0, 0xf
   12792:      	or	t2, a5, a4
   12796:      	srliw	a5, a0, 0x13
   1279a:      	slli	s1, a0, 0xd
   1279e:      	mv	a3, a0
   127a0:      	or	a7, s1, a5
   127a4:      	mv	a4, s0
   127a6:      	srliw	s1, s0, 0x11
   127aa:      	slli	s0, s0, 0xf
   127ac:      	or	s2, s0, s1
   127b0:      	srliw	s1, a4, 0x13
   127b4:      	slli	a0, a4, 0xd
   127b8:      	or	a0, a0, s1
   127ba:      	srliw	s1, a4, 0x7
   127be:      	slli	a1, a4, 0x19
   127c2:      	or	a1, a1, s1
   127c4:      	srliw	s1, a4, 0x12
   127c8:      	slli	a2, a4, 0xe
   127cc:      	mv	t4, a4
   127ce:      	or	a2, a2, s1
   127d0:      	mv	a5, a3
   127d2:      	srliw	s1, a3, 0x7
   127d6:      	slli	a3, a3, 0x19
   127d8:      	or	a3, a3, s1
   127da:      	srliw	s1, a5, 0x12
   127de:      	slli	a4, a5, 0xe
   127e2:      	mv	s0, a5
   127e4:      	or	a4, a4, s1
   127e6:      	add	a6, a6, t1
   127e8:      	add	t0, t0, s3
   127ea:      	xor	a5, t2, a7
   127ee:      	xor	a0, s2, a0
   127f2:      	xor	a1, a1, a2
   127f4:      	xor	a3, a3, a4
   127f6:      	add	s7, a6, t6
   127fa:      	add	t0, t0, a6
   127fc:      	sd	s0, 0x1b8(sp)
   127fe:      	srliw	a2, s0, 0xa
   12802:      	xor	a2, a2, a5
   12804:      	sd	t4, 0x178(sp)
   12806:      	srliw	a4, t4, 0xa
   1280a:      	xor	a0, a0, a4
   1280c:      	srliw	a4, t4, 0x3
   12810:      	xor	a1, a1, a4
   12812:      	sd	a1, 0x60(sp)
   12814:      	srliw	a1, s0, 0x3
   12818:      	xor	a1, a1, a3
   1281a:      	sd	a1, 0x68(sp)
   1281c:      	ld	a7, 0x30(sp)
   1281e:      	ld	a1, 0x158(sp)
   12820:      	add	a7, a7, a1
   12822:      	add	a7, a7, t5
   12824:      	add	a7, a7, a2
   12826:      	ld	a1, 0xe8(sp)
   12828:      	ld	a2, 0xb0(sp)
   1282a:      	add	a1, a1, a2
   1282c:      	add	a1, a1, s9
   1282e:      	add	s2, a1, a0
   12832:      	ld	a0, 0x1d0(sp)
   12834:      	add	a0, a0, s11
   12836:      	sd	a0, 0x1d0(sp)
   12838:      	xor	a0, t3, s11
   1283c:      	and	a0, s7, a0
   12840:      	xor	a0, a0, s11
   12844:      	srliw	a1, s7, 0x6
   12848:      	slli	a2, s7, 0x1a
   1284c:      	or	a1, a1, a2
   1284e:      	srliw	a2, s7, 0xb
   12852:      	slli	a3, s7, 0x15
   12856:      	or	a3, a3, a2
   12858:      	srliw	a2, s7, 0x19
   1285c:      	slli	a4, s7, 0x7
   12860:      	or	a6, a4, a2
   12864:      	srliw	a4, t0, 0x2
   12868:      	slli	a5, t0, 0x1e
   1286c:      	or	a4, a4, a5
   1286e:      	srliw	a5, t0, 0xd
   12872:      	slli	s1, t0, 0x13
   12876:      	or	a5, a5, s1
   12878:      	srliw	s1, t0, 0x16
   1287c:      	slli	s0, t0, 0xa
   12880:      	or	s0, s0, s1
   12882:      	xor	s1, s4, ra
   12886:      	and	s1, t0, s1
   1288a:      	mv	s9, t0
   1288c:      	and	a2, s4, ra
   12890:      	xor	t6, s1, a2
   12894:      	add	t5, s6, a0
   12898:      	xor	a1, a1, a3
   1289a:      	xor	a2, a4, a5
   1289e:      	mv	a5, a7
   128a0:      	srliw	a0, a7, 0x11
   128a4:      	slli	a3, a7, 0xf
   128a8:      	or	t0, a3, a0
   128ac:      	srliw	a0, a7, 0x13
   128b0:      	slli	a3, a7, 0xd
   128b4:      	or	a7, a3, a0
   128b8:      	srliw	a0, s2, 0x11
   128bc:      	slli	a3, s2, 0xf
   128c0:      	or	t1, a3, a0
   128c4:      	srliw	a0, s2, 0x13
   128c8:      	slli	s1, s2, 0xd
   128cc:      	or	s1, s1, a0
   128ce:      	srliw	a0, a5, 0x7
   128d2:      	slli	a3, a5, 0x19
   128d6:      	or	t2, a3, a0
   128da:      	srliw	a0, a5, 0x12
   128de:      	slli	a4, a5, 0xe
   128e2:      	mv	a3, a5
   128e4:      	or	a0, a0, a4
   128e6:      	xor	a6, a1, a6
   128ea:      	lui	a1, 0x550c8
   128ee:      	addi	a4, a1, -0x23d
   128f2:      	add	a4, a4, t5
   128f4:      	xor	a2, a2, s0
   128f6:      	srliw	a1, s2, 0x7
   128fa:      	slli	a5, s2, 0x19
   128fe:      	or	a1, a1, a5
   12900:      	srliw	a5, s2, 0x12
   12904:      	slli	s0, s2, 0xe
   12908:      	or	a5, a5, s0
   1290a:      	xor	s0, t0, a7
   1290e:      	xor	s1, t1, s1
   12912:      	xor	a0, t2, a0
   12916:      	add	a4, a4, a6
   12918:      	add	a2, a2, t6
   1291a:      	xor	a6, a1, a5
   1291e:      	mv	a1, a3
   12920:      	sd	a3, 0x98(sp)
   12922:      	srliw	a3, a3, 0xa
   12926:      	xor	a3, a3, s0
   12928:      	srliw	a5, s2, 0xa
   1292c:      	xor	a5, a5, s1
   1292e:      	srliw	s1, a1, 0x3
   12932:      	xor	a0, a0, s1
   12934:      	sd	a0, 0x30(sp)
   12936:      	add	t1, a4, s10
   1293a:      	add	t6, a2, a4
   1293e:      	srliw	a0, s2, 0x3
   12942:      	sd	s2, 0xb0(sp)
   12944:      	xor	a0, a6, a0
   12948:      	sd	a0, 0x28(sp)
   1294a:      	ld	a0, 0xa0(sp)
   1294c:      	ld	a1, 0x128(sp)
   1294e:      	add	a0, a0, a1
   12950:      	add	a0, a0, s5
   12952:      	add	s0, a0, a3
   12956:      	ld	a0, 0x168(sp)
   12958:      	ld	a1, 0xa8(sp)
   1295a:      	add	a0, a0, a1
   1295c:      	ld	a1, 0x1c0(sp)
   1295e:      	add	a0, a0, a1
   12960:      	add	s8, a0, a5
   12964:      	ld	a6, 0x148(sp)
   12966:      	add	a6, a6, t3
   12968:      	sd	a6, 0x10(sp)
   1296a:      	xor	a0, s7, t3
   1296e:      	and	a0, t1, a0
   12972:      	xor	t4, a0, t3
   12976:      	srliw	a0, t1, 0x6
   1297a:      	slli	a2, t1, 0x1a
   1297e:      	or	s5, a2, a0
   12982:      	srliw	a0, t1, 0xb
   12986:      	slli	a3, t1, 0x15
   1298a:      	or	a4, a3, a0
   1298e:      	srliw	a0, t1, 0x19
   12992:      	slli	a3, t1, 0x7
   12996:      	or	a7, a3, a0
   1299a:      	srliw	a0, t6, 0x2
   1299e:      	slli	a3, t6, 0x1e
   129a2:      	or	a0, a0, a3
   129a4:      	srliw	a3, t6, 0xd
   129a8:      	slli	a5, t6, 0x13
   129ac:      	or	a2, a5, a3
   129b0:      	srliw	a3, t6, 0x16
   129b4:      	slli	a5, t6, 0xa
   129b8:      	or	s3, a5, a3
   129bc:      	mv	a6, s9
   129be:      	xor	a3, s9, s4
   129c2:      	and	a3, t6, a3
   129c6:      	and	a5, s9, s4
   129ca:      	xor	t2, a3, a5
   129ce:      	mv	a1, s0
   129d0:      	srliw	a3, s0, 0x11
   129d4:      	slli	a5, s0, 0xf
   129d8:      	or	s6, a5, a3
   129dc:      	srliw	a3, s0, 0x13
   129e0:      	slli	a5, s0, 0xd
   129e4:      	or	t3, a5, a3
   129e8:      	srliw	a3, s8, 0x11
   129ec:      	slli	a5, s8, 0xf
   129f0:      	or	t5, a5, a3
   129f4:      	srliw	a3, s8, 0x13
   129f8:      	slli	a5, s8, 0xd
   129fc:      	or	t0, a5, a3
   12a00:      	srliw	a3, s0, 0x7
   12a04:      	slli	s0, s0, 0x19
   12a06:      	or	s10, s0, a3
   12a0a:      	srliw	a5, a1, 0x12
   12a0e:      	slli	a3, a1, 0xe
   12a12:      	or	s0, a3, a5
   12a16:      	ld	s9, 0x1d0(sp)
   12a18:      	add	s9, s9, t4
   12a1a:      	xor	a4, s5, a4
   12a1e:      	xor	a2, a2, a0
   12a20:      	srliw	a0, s8, 0x7
   12a24:      	slli	a5, s8, 0x19
   12a28:      	or	a0, a0, a5
   12a2a:      	srliw	a5, s8, 0x12
   12a2e:      	slli	s1, s8, 0xe
   12a32:      	or	a5, a5, s1
   12a34:      	xor	s1, s6, t3
   12a38:      	xor	a3, t5, t0
   12a3c:      	xor	t0, s10, s0
   12a40:      	xor	a4, a4, a7
   12a44:      	lui	s0, 0x72be6
   12a48:      	addi	s0, s0, -0x28c
   12a4c:      	add	s0, s0, s9
   12a4e:      	xor	a2, a2, s3
   12a52:      	xor	a0, a0, a5
   12a54:      	sd	a1, 0xa0(sp)
   12a56:      	srliw	a5, a1, 0xa
   12a5a:      	xor	a5, a5, s1
   12a5c:      	srliw	s1, s8, 0xa
   12a60:      	xor	a7, a3, s1
   12a64:      	srliw	s1, a1, 0x3
   12a68:      	xor	a3, t0, s1
   12a6c:      	sd	a3, 0x20(sp)
   12a6e:      	add	a4, a4, s0
   12a70:      	add	a2, a2, t2
   12a72:      	srliw	a3, s8, 0x3
   12a76:      	sd	s8, 0xa8(sp)
   12a78:      	xor	a0, a0, a3
   12a7a:      	sd	a0, 0x18(sp)
   12a7c:      	ld	a0, 0x40(sp)
   12a7e:      	ld	a1, 0x130(sp)
   12a80:      	add	a0, a0, a1
   12a82:      	ld	a3, 0x178(sp)
   12a84:      	add	a0, a0, a3
   12a86:      	add	t0, a0, a5
   12a8a:      	ld	a0, 0x50(sp)
   12a8c:      	ld	a1, 0x140(sp)
   12a8e:      	add	a0, a0, a1
   12a90:      	ld	a3, 0x1c8(sp)
   12a92:      	add	a0, a0, a3
   12a94:      	add	a3, a0, a7
   12a98:      	add	t4, a4, ra
   12a9c:      	add	s3, a2, a4
   12aa0:      	srliw	a0, t0, 0x7
   12aa4:      	slli	a1, t0, 0x19
   12aa8:      	or	a7, a1, a0
   12aac:      	srliw	a1, t0, 0x12
   12ab0:      	slli	a2, t0, 0xe
   12ab4:      	or	t3, a2, a1
   12ab8:      	mv	a0, a3
   12aba:      	srliw	a2, a3, 0x11
   12abe:      	slli	a3, a3, 0xf
   12ac0:      	or	t5, a3, a2
   12ac4:      	srliw	a3, a0, 0x13
   12ac8:      	slli	a4, a0, 0xd
   12acc:      	mv	t2, a0
   12ace:      	or	s6, a4, a3
   12ad2:      	srliw	a4, t0, 0x11
   12ad6:      	slli	a5, t0, 0xf
   12ada:      	or	s9, a5, a4
   12ade:      	srliw	a5, t0, 0x13
   12ae2:      	slli	s1, t0, 0xd
   12ae6:      	or	s11, s1, a5
   12aea:      	ld	a0, 0xc0(sp)
   12aec:      	add	s5, a0, s7
   12af0:      	xor	s1, t1, s7
   12af4:      	and	s1, t4, s1
   12af8:      	xor	s10, s1, s7
   12afc:      	srliw	s0, t4, 0x6
   12b00:      	slli	a0, t4, 0x1a
   12b04:      	or	a0, a0, s0
   12b06:      	srliw	s0, t4, 0xb
   12b0a:      	slli	a1, t4, 0x15
   12b0e:      	or	a1, a1, s0
   12b10:      	srliw	s0, t4, 0x19
   12b14:      	slli	a2, t4, 0x7
   12b18:      	or	s7, a2, s0
   12b1c:      	srliw	s0, s3, 0x2
   12b20:      	slli	a3, s3, 0x1e
   12b24:      	or	a3, a3, s0
   12b26:      	srliw	s0, s3, 0xd
   12b2a:      	slli	a4, s3, 0x13
   12b2e:      	or	a4, a4, s0
   12b30:      	srliw	s0, s3, 0x16
   12b34:      	slli	a5, s3, 0xa
   12b38:      	or	a5, a5, s0
   12b3a:      	sd	a6, 0x8(sp)
   12b3c:      	xor	s0, t6, a6
   12b40:      	and	s0, s3, s0
   12b44:      	and	a2, t6, a6
   12b48:      	xor	ra, s0, a2
   12b4c:      	mv	s1, t2
   12b4e:      	srliw	s0, t2, 0x7
   12b52:      	slli	a2, t2, 0x19
   12b56:      	or	a2, a2, s0
   12b58:      	srliw	s0, t2, 0x12
   12b5c:      	slli	t2, t2, 0xe
   12b5e:      	sd	s1, 0x148(sp)
   12b60:      	or	s0, t2, s0
   12b64:      	xor	a7, a7, t3
   12b68:      	xor	t2, t5, s6
   12b6c:      	xor	t3, s9, s11
   12b70:      	ld	a6, 0x10(sp)
   12b72:      	add	a6, a6, s10
   12b74:      	xor	a0, a0, a1
   12b76:      	xor	a3, a3, a4
   12b78:      	xor	a2, a2, s0
   12b7a:      	sd	t0, 0x1d0(sp)
   12b7c:      	srliw	a1, t0, 0x3
   12b80:      	xor	a1, a7, a1
   12b84:      	sd	a1, 0x40(sp)
   12b86:      	srliw	a1, s1, 0xa
   12b8a:      	xor	a1, t2, a1
   12b8e:      	srliw	a4, t0, 0xa
   12b92:      	xor	t5, t3, a4
   12b96:      	xor	a0, a0, s7
   12b9a:      	lui	a4, 0x80deb
   12b9e:      	addi	a4, a4, 0x1fe
   12ba2:      	add	a4, a4, a6
   12ba4:      	xor	a3, a3, a5
   12ba6:      	srliw	a5, s1, 0x3
   12baa:      	xor	a2, a2, a5
   12bac:      	sd	a2, 0x10(sp)
   12bae:      	ld	a2, 0xd8(sp)
   12bb0:      	ld	a5, 0x48(sp)
   12bb2:      	add	a2, a2, a5
   12bb4:      	ld	a5, 0x1b8(sp)
   12bb6:      	add	a2, a2, a5
   12bb8:      	add	a5, a2, a1
   12bbc:      	ld	a1, 0x38(sp)
   12bbe:      	ld	a2, 0x138(sp)
   12bc0:      	add	a1, a1, a2
   12bc2:      	add	a1, a1, s2
   12bc4:      	add	s2, a1, t5
   12bc8:      	add	a0, a0, a4
   12bca:      	add	a3, a3, ra
   12bcc:      	add	t2, a0, s4
   12bd0:      	add	t5, a3, a0
   12bd4:      	srliw	a0, s2, 0x11
   12bd8:      	slli	a1, s2, 0xf
   12bdc:      	or	s11, a1, a0
   12be0:      	srliw	a1, s2, 0x13
   12be4:      	slli	a2, s2, 0xd
   12be8:      	or	a6, a2, a1
   12bec:      	srliw	a2, a5, 0x11
   12bf0:      	slli	a3, a5, 0xf
   12bf4:      	or	s1, a3, a2
   12bf8:      	srliw	a3, a5, 0x13
   12bfc:      	slli	a4, a5, 0xd
   12c00:      	or	s4, a4, a3
   12c04:      	mv	a1, a5
   12c06:      	srliw	a4, a5, 0x7
   12c0a:      	slli	a5, a5, 0x19
   12c0c:      	or	s6, a5, a4
   12c10:      	srliw	a0, a1, 0x12
   12c14:      	slli	a5, a1, 0xe
   12c18:      	mv	t0, a1
   12c1a:      	or	s7, a5, a0
   12c1e:      	srliw	a0, s2, 0x7
   12c22:      	slli	a1, s2, 0x19
   12c26:      	or	s9, a1, a0
   12c2a:      	srliw	a1, s2, 0x12
   12c2e:      	slli	a2, s2, 0xe
   12c32:      	or	a7, a2, a1
   12c36:      	xor	ra, s11, a6
   12c3a:      	xor	s11, s1, s4
   12c3e:      	ld	t3, 0x170(sp)
   12c40:      	add	t3, t3, t1
   12c42:      	xor	a4, t4, t1
   12c46:      	and	a4, t2, a4
   12c4a:      	xor	t1, a4, t1
   12c4e:      	srliw	a5, t2, 0x6
   12c52:      	slli	s0, t2, 0x1a
   12c56:      	or	a5, a5, s0
   12c58:      	srliw	s0, t2, 0xb
   12c5c:      	slli	s1, t2, 0x15
   12c60:      	or	s0, s0, s1
   12c62:      	srliw	s1, t2, 0x19
   12c66:      	slli	a0, t2, 0x7
   12c6a:      	or	s4, a0, s1
   12c6e:      	srliw	s1, t5, 0x2
   12c72:      	slli	a0, t5, 0x1e
   12c76:      	or	a0, a0, s1
   12c78:      	srliw	s1, t5, 0xd
   12c7c:      	slli	a1, t5, 0x13
   12c80:      	or	a1, a1, s1
   12c82:      	srliw	s1, t5, 0x16
   12c86:      	slli	a3, t5, 0xa
   12c8a:      	or	a6, a3, s1
   12c8e:      	xor	s1, s3, t6
   12c92:      	and	s1, t5, s1
   12c96:      	and	a2, s3, t6
   12c9a:      	xor	a2, a2, s1
   12c9c:      	xor	s1, s6, s7
   12ca0:      	xor	a4, s9, a7
   12ca4:      	sd	s2, 0xc0(sp)
   12ca6:      	srliw	a3, s2, 0xa
   12caa:      	xor	ra, ra, a3
   12cae:      	srliw	a3, t0, 0xa
   12cb2:      	xor	a3, s11, a3
   12cb6:      	add	a7, s5, t1
   12cba:      	xor	a5, a5, s0
   12cbc:      	xor	a0, a0, a1
   12cbe:      	srliw	a1, t0, 0x3
   12cc2:      	mv	s5, t0
   12cc4:      	sd	t0, 0x0(sp)
   12cc6:      	xor	a1, a1, s1
   12cc8:      	sd	a1, 0x38(sp)
   12cca:      	srliw	a1, s2, 0x3
   12cce:      	xor	a1, a1, a4
   12cd0:      	sd	a1, 0x48(sp)
   12cd2:      	ld	a1, 0xc8(sp)
   12cd4:      	ld	a4, 0x58(sp)
   12cd6:      	add	a1, a1, a4
   12cd8:      	add	a1, a1, s8
   12cda:      	add	s10, a1, ra
   12cde:      	ld	a1, 0xd0(sp)
   12ce0:      	ld	a4, 0x78(sp)
   12ce2:      	add	a1, a1, a4
   12ce4:      	ld	s2, 0x98(sp)
   12ce6:      	add	a1, a1, s2
   12ce8:      	add	s1, a1, a3
   12cec:      	xor	a1, a5, s4
   12cf0:      	lui	a3, 0x9bdc0
   12cf4:      	addi	a3, a3, 0x6a7
   12cf8:      	add	a3, a3, a7
   12cfa:      	xor	a0, a0, a6
   12cfe:      	add	a1, a1, a3
   12d00:      	add	a0, a0, a2
   12d02:      	srliw	a2, s10, 0x11
   12d06:      	slli	a3, s10, 0xf
   12d0a:      	or	a2, a2, a3
   12d0c:      	srliw	a3, s10, 0x13
   12d10:      	slli	a4, s10, 0xd
   12d14:      	or	a3, a3, a4
   12d16:      	srliw	a4, s1, 0x11
   12d1a:      	slli	a5, s1, 0xf
   12d1e:      	or	a4, a4, a5
   12d20:      	srliw	s0, s1, 0x13
   12d24:      	slli	a5, s1, 0xd
   12d28:      	or	a5, a5, s0
   12d2a:      	ld	s4, 0x8(sp)
   12d2c:      	add	s4, s4, a1
   12d2e:      	add	t0, a0, a1
   12d32:      	mv	a0, s1
   12d34:      	srliw	a1, s1, 0x7
   12d38:      	slli	s1, s1, 0x19
   12d3a:      	or	a1, a1, s1
   12d3c:      	srliw	s1, a0, 0x12
   12d40:      	slli	s0, a0, 0xe
   12d44:      	or	s0, s0, s1
   12d46:      	xor	a2, a2, a3
   12d48:      	xor	a4, a4, a5
   12d4a:      	xor	s6, a1, s0
   12d4e:      	srliw	a3, s10, 0xa
   12d52:      	sd	s10, 0x50(sp)
   12d54:      	xor	s7, a2, a3
   12d58:      	srliw	a3, a0, 0xa
   12d5c:      	mv	t1, a0
   12d5e:      	xor	a7, a4, a3
   12d62:      	ld	s11, 0x180(sp)
   12d64:      	add	s11, s11, t4
   12d66:      	xor	a5, t2, t4
   12d6a:      	and	a5, s4, a5
   12d6e:      	xor	s9, a5, t4
   12d72:      	srliw	s1, s4, 0x6
   12d76:      	slli	s0, s4, 0x1a
   12d7a:      	or	s0, s0, s1
   12d7c:      	srliw	s1, s4, 0xb
   12d80:      	slli	a3, s4, 0x15
   12d84:      	or	a3, a3, s1
   12d86:      	srliw	s1, s4, 0x19
   12d8a:      	slli	a4, s4, 0x7
   12d8e:      	or	a4, a4, s1
   12d90:      	srliw	s1, t0, 0x2
   12d94:      	slli	a0, t0, 0x1e
   12d98:      	or	a0, a0, s1
   12d9a:      	srliw	s1, t0, 0xd
   12d9e:      	slli	a2, t0, 0x13
   12da2:      	or	a2, a2, s1
   12da4:      	srliw	s1, t0, 0x16
   12da8:      	slli	a1, t0, 0xa
   12dac:      	or	a1, a1, s1
   12dae:      	xor	s1, t5, s3
   12db2:      	and	s1, t0, s1
   12db6:      	and	a5, t5, s3
   12dba:      	xor	a6, s1, a5
   12dbe:      	srliw	s1, t1, 0x3
   12dc2:      	mv	ra, t1
   12dc4:      	sd	t1, 0x58(sp)
   12dc6:      	xor	a5, s6, s1
   12dca:      	sd	a5, 0x8(sp)
   12dcc:      	ld	s1, 0x1c0(sp)
   12dce:      	ld	a5, 0x70(sp)
   12dd0:      	add	s1, s1, a5
   12dd2:      	ld	a5, 0x148(sp)
   12dd4:      	add	s1, s1, a5
   12dd6:      	add	s7, s7, s1
   12dd8:      	ld	s1, 0x120(sp)
   12dda:      	ld	a5, 0x88(sp)
   12ddc:      	add	s1, s1, a5
   12dde:      	ld	s8, 0xa0(sp)
   12de0:      	add	t4, s1, s8
   12de4:      	add	t1, t4, a7
   12de8:      	add	t3, t3, s9
   12dea:      	xor	a3, a3, s0
   12dec:      	xor	a0, a0, a2
   12dee:      	xor	a3, a3, a4
   12df0:      	lui	a2, 0xc19bf
   12df4:      	addi	a2, a2, 0x174
   12df8:      	add	a2, a2, t3
   12dfa:      	xor	a0, a0, a1
   12dfc:      	srliw	a1, s7, 0x11
   12e00:      	slli	a4, s7, 0xf
   12e04:      	or	a1, a1, a4
   12e06:      	srliw	a4, s7, 0x13
   12e0a:      	slli	s1, s7, 0xd
   12e0e:      	sd	s7, 0x180(sp)
   12e10:      	or	a4, a4, s1
   12e12:      	srliw	s1, t1, 0x11
   12e16:      	slli	s0, t1, 0xf
   12e1a:      	or	s0, s0, s1
   12e1c:      	srliw	s1, t1, 0x13
   12e20:      	slli	a5, t1, 0xd
   12e24:      	or	a5, a5, s1
   12e26:      	add	a2, a2, a3
   12e28:      	add	a0, a0, a6
   12e2a:      	xor	a1, a1, a4
   12e2c:      	xor	a5, a5, s0
   12e2e:      	add	s9, a2, t6
   12e32:      	add	t4, a0, a2
   12e36:      	srliw	a0, s7, 0xa
   12e3a:      	xor	a0, a0, a1
   12e3c:      	srliw	a1, t1, 0xa
   12e40:      	sd	t1, 0x70(sp)
   12e42:      	xor	a1, a1, a5
   12e44:      	ld	a3, 0x1c8(sp)
   12e46:      	ld	a2, 0x60(sp)
   12e48:      	add	a3, a3, a2
   12e4a:      	add	a3, a3, s5
   12e4c:      	add	a7, a3, a0
   12e50:      	ld	a0, 0x80(sp)
   12e52:      	ld	a2, 0xb8(sp)
   12e54:      	add	a0, a0, a2
   12e56:      	ld	a2, 0x1d0(sp)
   12e58:      	add	a0, a0, a2
   12e5a:      	add	s6, a0, a1
   12e5e:      	ld	t3, 0x188(sp)
   12e60:      	add	t3, t3, t2
   12e62:      	xor	a0, s4, t2
   12e66:      	and	a0, s9, a0
   12e6a:      	xor	t2, a0, t2
   12e6e:      	srliw	a3, s9, 0x6
   12e72:      	slli	a4, s9, 0x1a
   12e76:      	or	a3, a3, a4
   12e78:      	srliw	a4, s9, 0xb
   12e7c:      	slli	a5, s9, 0x15
   12e80:      	or	a4, a4, a5
   12e82:      	srliw	a5, s9, 0x19
   12e86:      	slli	s1, s9, 0x7
   12e8a:      	or	a5, a5, s1
   12e8c:      	srliw	s1, t4, 0x2
   12e90:      	slli	a2, t4, 0x1e
   12e94:      	or	a2, a2, s1
   12e96:      	srliw	s1, t4, 0xd
   12e9a:      	slli	s0, t4, 0x13
   12e9e:      	or	s0, s0, s1
   12ea0:      	srliw	s1, t4, 0x16
   12ea4:      	slli	a1, t4, 0xa
   12ea8:      	or	a1, a1, s1
   12eaa:      	xor	s1, t0, t5
   12eae:      	and	s1, t4, s1
   12eb2:      	and	a0, t0, t5
   12eb6:      	xor	t6, s1, a0
   12eba:      	add	t2, t2, s11
   12ebc:      	xor	a3, a3, a4
   12ebe:      	xor	a2, a2, s0
   12ec0:      	srliw	a4, s6, 0x11
   12ec4:      	slli	s1, s6, 0xf
   12ec8:      	or	a4, a4, s1
   12eca:      	srliw	s1, s6, 0x13
   12ece:      	slli	s0, s6, 0xd
   12ed2:      	or	s0, s0, s1
   12ed4:      	xor	a3, a3, a5
   12ed6:      	lui	a5, 0xe49b7
   12eda:      	addi	a5, a5, -0x63f
   12ede:      	add	a5, a5, t2
   12ee0:      	xor	a1, a1, a2
   12ee2:      	srliw	a2, a7, 0x11
   12ee6:      	slli	s1, a7, 0xf
   12eea:      	or	a2, a2, s1
   12eec:      	srliw	s1, a7, 0x13
   12ef0:      	slli	a0, a7, 0xd
   12ef4:      	sd	a7, 0x170(sp)
   12ef6:      	or	a0, a0, s1
   12ef8:      	xor	a4, a4, s0
   12efa:      	add	a3, a3, a5
   12efc:      	add	a1, a1, t6
   12efe:      	xor	a0, a0, a2
   12f00:      	srliw	a2, s6, 0xa
   12f04:      	xor	a2, a2, a4
   12f06:      	add	s3, s3, a3
   12f08:      	add	t2, a1, a3
   12f0c:      	srliw	a1, a7, 0xa
   12f10:      	xor	a0, a0, a1
   12f12:      	ld	a1, 0x178(sp)
   12f14:      	ld	a3, 0x68(sp)
   12f16:      	add	a1, a1, a3
   12f18:      	ld	a3, 0xc0(sp)
   12f1a:      	add	a1, a1, a3
   12f1c:      	add	s5, a1, a2
   12f20:      	ld	a1, 0x1b8(sp)
   12f22:      	ld	a2, 0x28(sp)
   12f24:      	add	a1, a1, a2
   12f26:      	add	a1, a1, ra
   12f28:      	add	s7, a1, a0
   12f2c:      	ld	t6, 0xe0(sp)
   12f2e:      	add	t6, t6, s4
   12f30:      	xor	a0, s9, s4
   12f34:      	and	a0, s3, a0
   12f38:      	xor	a0, a0, s4
   12f3c:      	srliw	a1, s3, 0x6
   12f40:      	slli	a2, s3, 0x1a
   12f44:      	or	a1, a1, a2
   12f46:      	srliw	a2, s3, 0xb
   12f4a:      	slli	a4, s3, 0x15
   12f4e:      	or	a2, a2, a4
   12f50:      	srliw	a4, s3, 0x19
   12f54:      	slli	s1, s3, 0x7
   12f58:      	or	s4, s1, a4
   12f5c:      	srliw	s1, t2, 0x2
   12f60:      	slli	a3, t2, 0x1e
   12f64:      	or	a3, a3, s1
   12f66:      	srliw	s1, t2, 0xd
   12f6a:      	slli	s0, t2, 0x13
   12f6e:      	or	s0, s0, s1
   12f70:      	srliw	s1, t2, 0x16
   12f74:      	slli	a5, t2, 0xa
   12f78:      	or	a5, a5, s1
   12f7a:      	xor	s1, t4, t0
   12f7e:      	and	s1, t2, s1
   12f82:      	and	a4, t4, t0
   12f86:      	xor	a7, s1, a4
   12f8a:      	srliw	s1, s5, 0x11
   12f8e:      	slli	s11, s5, 0xf
   12f92:      	or	s11, s11, s1
   12f96:      	srliw	s1, s5, 0x13
   12f9a:      	slli	ra, s5, 0xd
   12f9e:      	or	s1, ra, s1
   12fa2:      	add	a0, a0, t3
   12fa4:      	xor	a1, a1, a2
   12fa6:      	xor	a3, a3, s0
   12fa8:      	srliw	a2, s7, 0x11
   12fac:      	slli	s0, s7, 0xf
   12fb0:      	or	a2, a2, s0
   12fb2:      	srliw	s0, s7, 0x13
   12fb6:      	slli	a4, s7, 0xd
   12fba:      	sd	s7, 0x188(sp)
   12fbc:      	or	a4, a4, s0
   12fbe:      	xor	s1, s11, s1
   12fc2:      	xor	a1, a1, s4
   12fc6:      	lui	s0, 0xefbe4
   12fca:      	addi	s0, s0, 0x786
   12fce:      	add	a0, a0, s0
   12fd0:      	xor	a3, a3, a5
   12fd2:      	xor	a2, a2, a4
   12fd4:      	srliw	a4, s5, 0xa
   12fd8:      	sd	s5, 0x78(sp)
   12fda:      	xor	a4, a4, s1
   12fdc:      	add	a0, a0, a1
   12fde:      	add	a1, a3, a7
   12fe2:      	srliw	a3, s7, 0xa
   12fe6:      	xor	a2, a2, a3
   12fe8:      	ld	a3, 0xb0(sp)
   12fea:      	ld	a5, 0x30(sp)
   12fec:      	add	a3, a3, a5
   12fee:      	add	a3, a3, s10
   12ff0:      	add	s10, a3, a4
   12ff4:      	add	t5, t5, a0
   12ff6:      	add	s11, a1, a0
   12ffa:      	ld	a0, 0x18(sp)
   12ffc:      	add	a0, a0, s2
   12ffe:      	add	a0, a0, t1
   13000:      	add	s2, a0, a2
   13004:      	srliw	a0, s10, 0x11
   13008:      	slli	a2, s10, 0xf
   1300c:      	or	a7, a2, a0
   13010:      	srliw	a2, s10, 0x13
   13014:      	slli	a3, s10, 0xd
   13018:      	or	ra, a3, a2
   1301c:      	ld	t3, 0x198(sp)
   1301e:      	add	t3, t3, s9
   13020:      	xor	a3, s3, s9
   13024:      	and	a3, t5, a3
   13028:      	xor	s9, a3, s9
   1302c:      	srliw	a5, t5, 0x6
   13030:      	slli	s1, t5, 0x1a
   13034:      	or	a5, a5, s1
   13036:      	srliw	s1, t5, 0xb
   1303a:      	slli	s0, t5, 0x15
   1303e:      	or	s0, s0, s1
   13040:      	srliw	s1, t5, 0x19
   13044:      	slli	a4, t5, 0x7
   13048:      	or	a4, a4, s1
   1304a:      	srliw	s1, s11, 0x2
   1304e:      	slli	a1, s11, 0x1e
   13052:      	or	a1, a1, s1
   13054:      	srliw	s1, s11, 0xd
   13058:      	slli	a0, s11, 0x13
   1305c:      	or	a0, a0, s1
   1305e:      	srliw	s1, s11, 0x16
   13062:      	slli	a2, s11, 0xa
   13066:      	or	a2, a2, s1
   13068:      	xor	s1, t2, t4
   1306c:      	and	s1, s11, s1
   13070:      	and	a3, t2, t4
   13074:      	xor	a3, a3, s1
   13076:      	srliw	s1, s2, 0x11
   1307a:      	slli	s4, s2, 0xf
   1307e:      	or	s4, s4, s1
   13082:      	srliw	s1, s2, 0x13
   13086:      	slli	s7, s2, 0xd
   1308a:      	sd	s2, 0x80(sp)
   1308c:      	or	s1, s7, s1
   13090:      	xor	a7, a7, ra
   13094:      	add	t6, t6, s9
   13096:      	xor	a5, a5, s0
   13098:      	xor	a0, a0, a1
   1309a:      	xor	a1, s4, s1
   1309e:      	srliw	s0, s10, 0xa
   130a2:      	xor	s9, a7, s0
   130a6:      	xor	a4, a4, a5
   130a8:      	lui	a5, 0xfc1a
   130ac:      	addi	a5, a5, -0x23a
   130b0:      	add	a5, a5, t6
   130b2:      	xor	a0, a0, a2
   130b4:      	srliw	a2, s2, 0xa
   130b8:      	xor	a1, a1, a2
   130ba:      	ld	a2, 0xa8(sp)
   130bc:      	ld	s1, 0x20(sp)
   130be:      	add	a2, a2, s1
   130c0:      	ld	s1, 0x180(sp)
   130c2:      	add	a2, a2, s1
   130c4:      	add	s9, s9, a2
   130c6:      	add	a4, a4, a5
   130c8:      	add	a3, a3, a0
   130ca:      	ld	a0, 0x10(sp)
   130cc:      	add	a0, a0, s8
   130ce:      	add	a0, a0, s6
   130d0:      	mv	s8, s6
   130d2:      	add	s2, a0, a1
   130d6:      	add	t0, t0, a4
   130d8:      	add	s4, a3, a4
   130dc:      	srliw	a1, s9, 0x11
   130e0:      	slli	a2, s9, 0xf
   130e4:      	or	a1, a1, a2
   130e6:      	srliw	a2, s9, 0x13
   130ea:      	slli	a3, s9, 0xd
   130ee:      	or	a2, a2, a3
   130f0:      	srliw	a3, s2, 0x11
   130f4:      	slli	a4, s2, 0xf
   130f8:      	or	t6, a4, a3
   130fc:      	srliw	a4, s2, 0x13
   13100:      	slli	s1, s2, 0xd
   13104:      	sd	s2, 0x88(sp)
   13106:      	or	ra, s1, a4
   1310a:      	xor	s7, a1, a2
   1310e:      	ld	t1, 0x190(sp)
   13110:      	add	t1, t1, s3
   13112:      	xor	a2, t5, s3
   13116:      	and	a2, t0, a2
   1311a:      	xor	s3, a2, s3
   1311e:      	srliw	s1, t0, 0x6
   13122:      	slli	a0, t0, 0x1a
   13126:      	or	a0, a0, s1
   13128:      	srliw	s1, t0, 0xb
   1312c:      	slli	s0, t0, 0x15
   13130:      	or	s0, s0, s1
   13132:      	srliw	s1, t0, 0x19
   13136:      	slli	a3, t0, 0x7
   1313a:      	or	a7, a3, s1
   1313e:      	srliw	s1, s4, 0x2
   13142:      	slli	a5, s4, 0x1e
   13146:      	or	a5, a5, s1
   13148:      	srliw	s1, s4, 0xd
   1314c:      	slli	a1, s4, 0x13
   13150:      	or	a1, a1, s1
   13152:      	srliw	s1, s4, 0x16
   13156:      	slli	a4, s4, 0xa
   1315a:      	or	a4, a4, s1
   1315c:      	xor	s1, s11, t2
   13160:      	and	s1, s4, s1
   13164:      	and	a2, s11, t2
   13168:      	xor	a2, a2, s1
   1316a:      	xor	s1, t6, ra
   1316e:      	srliw	a3, s9, 0xa
   13172:      	xor	a3, s7, a3
   13176:      	add	t3, t3, s3
   13178:      	xor	a0, a0, s0
   1317a:      	xor	a1, a1, a5
   1317c:      	srliw	a5, s2, 0xa
   13180:      	xor	a5, a5, s1
   13182:      	ld	s1, 0x40(sp)
   13184:      	ld	s0, 0x148(sp)
   13186:      	add	s1, s1, s0
   13188:      	ld	s0, 0x170(sp)
   1318a:      	add	s1, s1, s0
   1318c:      	add	s3, s1, a3
   13190:      	xor	a0, a0, a7
   13194:      	lui	a3, 0x240ca
   13198:      	addi	a3, a3, 0x1cc
   1319c:      	add	a3, a3, t3
   1319e:      	xor	a1, a1, a4
   131a0:      	ld	a4, 0x38(sp)
   131a2:      	ld	s1, 0x1d0(sp)
   131a4:      	add	a4, a4, s1
   131a6:      	add	a4, a4, s5
   131a8:      	add	a4, a4, a5
   131aa:      	add	a0, a0, a3
   131ac:      	add	a1, a1, a2
   131ae:      	add	t6, a0, t4
   131b2:      	add	t3, a1, a0
   131b6:      	srliw	a0, s3, 0x11
   131ba:      	slli	a1, s3, 0xf
   131be:      	or	a0, a0, a1
   131c0:      	srliw	a1, s3, 0x13
   131c4:      	slli	a3, s3, 0xd
   131c8:      	or	a1, a1, a3
   131ca:      	srliw	a3, a4, 0x11
   131ce:      	slli	a5, a4, 0xf
   131d2:      	or	a3, a3, a5
   131d4:      	srliw	a5, a4, 0x13
   131d8:      	slli	s1, a4, 0xd
   131dc:      	mv	t4, a4
   131de:      	sd	a4, 0xe0(sp)
   131e0:      	or	a5, a5, s1
   131e2:      	xor	s7, a0, a1
   131e6:      	xor	a7, a3, a5
   131ea:      	ld	a6, 0x150(sp)
   131ec:      	add	a6, a6, t5
   131ee:      	xor	a1, t0, t5
   131f2:      	and	a1, t6, a1
   131f6:      	xor	t5, a1, t5
   131fa:      	srliw	a5, t6, 0x6
   131fe:      	slli	a2, t6, 0x1a
   13202:      	or	a2, a2, a5
   13204:      	srliw	a5, t6, 0xb
   13208:      	slli	s0, t6, 0x15
   1320c:      	or	a5, a5, s0
   1320e:      	srliw	s1, t6, 0x19
   13212:      	slli	a4, t6, 0x7
   13216:      	or	ra, a4, s1
   1321a:      	srliw	s1, t3, 0x2
   1321e:      	slli	s0, t3, 0x1e
   13222:      	or	s0, s0, s1
   13224:      	srliw	s1, t3, 0xd
   13228:      	slli	a3, t3, 0x13
   1322c:      	or	a3, a3, s1
   1322e:      	srliw	s1, t3, 0x16
   13232:      	slli	a0, t3, 0xa
   13236:      	or	a0, a0, s1
   13238:      	xor	s1, s4, s11
   1323c:      	and	s1, t3, s1
   13240:      	and	a1, s4, s11
   13244:      	xor	a1, a1, s1
   13246:      	srliw	s1, s3, 0xa
   1324a:      	xor	s1, s7, s1
   1324e:      	srliw	a4, t4, 0xa
   13252:      	xor	a4, a7, a4
   13256:      	add	t1, t1, t5
   13258:      	xor	a2, a2, a5
   1325a:      	xor	a3, a3, s0
   1325c:      	ld	s7, 0x0(sp)
   1325e:      	ld	a5, 0x48(sp)
   13260:      	add	a5, a5, s7
   13262:      	ld	s0, 0x188(sp)
   13264:      	add	a5, a5, s0
   13266:      	add	a5, a5, s1
   13268:      	sd	a5, 0x190(sp)
   1326a:      	ld	s6, 0x8(sp)
   1326c:      	ld	t4, 0xc0(sp)
   1326e:      	add	s6, s6, t4
   13270:      	add	s6, s6, s10
   13272:      	add	a4, a4, s6
   13274:      	sd	a4, 0x198(sp)
   13276:      	xor	a2, a2, ra
   1327a:      	lui	a4, 0x2de93
   1327e:      	addi	a4, a4, -0x391
   13282:      	add	a4, a4, t1
   13284:      	xor	a0, a0, a3
   13286:      	add	a2, a2, a4
   13288:      	add	a0, a0, a1
   1328a:      	add	a3, a2, t2
   1328e:      	add	t1, a0, a2
   13292:      	ld	a7, 0xf0(sp)
   13294:      	add	a7, a7, t0
   13296:      	xor	a0, t6, t0
   1329a:      	and	a0, a0, a3
   1329c:      	xor	t0, a0, t0
   132a0:      	srliw	a1, a3, 0x6
   132a4:      	slli	a2, a3, 0x1a
   132a8:      	or	t2, a2, a1
   132ac:      	srliw	a2, a3, 0xb
   132b0:      	slli	a4, a3, 0x15
   132b4:      	or	a2, a2, a4
   132b6:      	srliw	a4, a3, 0x19
   132ba:      	slli	a5, a3, 0x7
   132be:      	or	a4, a4, a5
   132c0:      	srliw	a5, t1, 0x2
   132c4:      	slli	s1, t1, 0x1e
   132c8:      	or	a5, a5, s1
   132ca:      	srliw	s1, t1, 0xd
   132ce:      	slli	s0, t1, 0x13
   132d2:      	or	s0, s0, s1
   132d4:      	srliw	s1, t1, 0x16
   132d8:      	slli	a0, t1, 0xa
   132dc:      	or	a0, a0, s1
   132de:      	xor	s1, t3, s4
   132e2:      	and	s1, t1, s1
   132e6:      	and	a1, t3, s4
   132ea:      	xor	a1, a1, s1
   132ec:      	add	a6, a6, t0
   132ee:      	xor	a2, t2, a2
   132f2:      	xor	a5, a5, s0
   132f4:      	xor	a2, a2, a4
   132f6:      	lui	a4, 0x4a748
   132fa:      	addi	a4, a4, 0x4aa
   132fe:      	add	a4, a4, a6
   13300:      	xor	a0, a0, a5
   13302:      	add	a2, a2, a4
   13304:      	add	a0, a0, a1
   13306:      	add	s11, s11, a2
   13308:      	add	a6, a0, a2
   1330c:      	ld	s6, 0x160(sp)
   1330e:      	add	s6, s6, t6
   13310:      	xor	a0, a3, t6
   13314:      	and	a0, s11, a0
   13318:      	xor	t0, a0, t6
   1331c:      	srliw	a2, s11, 0x6
   13320:      	slli	a4, s11, 0x1a
   13324:      	or	t2, a4, a2
   13328:      	srliw	a4, s11, 0xb
   1332c:      	slli	a5, s11, 0x15
   13330:      	or	a4, a4, a5
   13332:      	srliw	a5, s11, 0x19
   13336:      	slli	a1, s11, 0x7
   1333a:      	or	a1, a1, a5
   1333c:      	srliw	a5, a6, 0x2
   13340:      	slli	s1, a6, 0x1e
   13344:      	or	a5, a5, s1
   13346:      	srliw	s1, a6, 0xd
   1334a:      	slli	s0, a6, 0x13
   1334e:      	or	s0, s0, s1
   13350:      	srliw	s1, a6, 0x16
   13354:      	slli	a0, a6, 0xa
   13358:      	or	a0, a0, s1
   1335a:      	xor	s1, t1, t3
   1335e:      	and	s1, a6, s1
   13362:      	and	a2, t1, t3
   13366:      	xor	a2, a2, s1
   13368:      	add	a7, a7, t0
   1336a:      	xor	a4, t2, a4
   1336e:      	xor	a5, a5, s0
   13370:      	xor	a1, a1, a4
   13372:      	lui	a4, 0x5cb0b
   13376:      	addi	a4, a4, -0x624
   1337a:      	add	a4, a4, a7
   1337c:      	xor	a0, a0, a5
   1337e:      	add	a1, a1, a4
   13380:      	add	a0, a0, a2
   13382:      	add	s4, s4, a1
   13384:      	add	t0, a0, a1
   13388:      	ld	a7, 0xe8(sp)
   1338a:      	add	a7, a7, a3
   1338c:      	xor	a1, s11, a3
   13390:      	and	a1, s4, a1
   13394:      	xor	t2, a1, a3
   13398:      	srliw	a2, s4, 0x6
   1339c:      	slli	a3, s4, 0x1a
   133a0:      	or	a2, a2, a3
   133a2:      	srliw	a3, s4, 0xb
   133a6:      	slli	a4, s4, 0x15
   133aa:      	or	a3, a3, a4
   133ac:      	srliw	a4, s4, 0x19
   133b0:      	slli	a5, s4, 0x7
   133b4:      	or	a4, a4, a5
   133b6:      	srliw	a5, t0, 0x2
   133ba:      	slli	a0, t0, 0x1e
   133be:      	or	a0, a0, a5
   133c0:      	srliw	a5, t0, 0xd
   133c4:      	slli	s0, t0, 0x13
   133c8:      	or	a5, a5, s0
   133ca:      	srliw	s1, t0, 0x16
   133ce:      	slli	s0, t0, 0xa
   133d2:      	or	s0, s0, s1
   133d4:      	xor	s1, a6, t1
   133d8:      	and	s1, t0, s1
   133dc:      	and	a1, a6, t1
   133e0:      	xor	a1, a1, s1
   133e2:      	add	t2, t2, s6
   133e4:      	xor	a2, a2, a3
   133e6:      	xor	a0, a0, a5
   133e8:      	xor	a2, a2, a4
   133ea:      	lui	a3, 0x76f99
   133ee:      	addi	a3, a3, -0x726
   133f2:      	add	a3, a3, t2
   133f4:      	xor	a0, a0, s0
   133f6:      	add	a2, a2, a3
   133f8:      	add	a0, a0, a1
   133fa:      	add	t3, t3, a2
   133fc:      	add	t2, a0, a2
   13400:      	ld	s6, 0x158(sp)
   13402:      	add	s6, s6, s11
   13404:      	xor	a0, s4, s11
   13408:      	and	a0, t3, a0
   1340c:      	xor	t5, a0, s11
   13410:      	srliw	a1, t3, 0x6
   13414:      	slli	a3, t3, 0x1a
   13418:      	or	a1, a1, a3
   1341a:      	srliw	a3, t3, 0xb
   1341e:      	slli	a5, t3, 0x15
   13422:      	or	a3, a3, a5
   13424:      	srliw	a5, t3, 0x19
   13428:      	slli	a4, t3, 0x7
   1342c:      	or	a4, a4, a5
   1342e:      	srliw	a5, t2, 0x2
   13432:      	slli	a2, t2, 0x1e
   13436:      	or	a2, a2, a5
   13438:      	srliw	a5, t2, 0xd
   1343c:      	slli	s0, t2, 0x13
   13440:      	or	a5, a5, s0
   13442:      	srliw	s1, t2, 0x16
   13446:      	slli	s0, t2, 0xa
   1344a:      	or	s0, s0, s1
   1344c:      	xor	s1, t0, a6
   13450:      	and	s1, t2, s1
   13454:      	and	a0, t0, a6
   13458:      	xor	a0, a0, s1
   1345a:      	add	a7, a7, t5
   1345c:      	xor	a1, a1, a3
   1345e:      	xor	a2, a2, a5
   13460:      	xor	a1, a1, a4
   13462:      	lui	a3, 0x983e5
   13466:      	addi	a3, a3, 0x152
   1346a:      	add	a3, a3, a7
   1346c:      	xor	a2, a2, s0
   1346e:      	add	a1, a1, a3
   13470:      	add	a0, a0, a2
   13472:      	add	s1, a1, t1
   13476:      	add	t1, a0, a1
   1347a:      	ld	a7, 0x168(sp)
   1347c:      	add	a7, a7, s4
   1347e:      	xor	a0, t3, s4
   13482:      	and	a0, a0, s1
   13484:      	xor	t5, a0, s4
   13488:      	srliw	a1, s1, 0x6
   1348c:      	slli	a2, s1, 0x1a
   13490:      	or	t6, a2, a1
   13494:      	srliw	a2, s1, 0xb
   13498:      	slli	a4, s1, 0x15
   1349c:      	or	a2, a2, a4
   1349e:      	srliw	a4, s1, 0x19
   134a2:      	slli	a5, s1, 0x7
   134a6:      	or	a4, a4, a5
   134a8:      	srliw	a5, t1, 0x2
   134ac:      	slli	a3, t1, 0x1e
   134b0:      	or	a3, a3, a5
   134b2:      	srliw	a5, t1, 0xd
   134b6:      	slli	s0, t1, 0x13
   134ba:      	or	a5, a5, s0
   134bc:      	srliw	s0, t1, 0x16
   134c0:      	slli	a0, t1, 0xa
   134c4:      	or	a0, a0, s0
   134c6:      	xor	s0, t2, t0
   134ca:      	and	s0, t1, s0
   134ce:      	and	a1, t2, t0
   134d2:      	xor	a1, a1, s0
   134d4:      	add	t5, t5, s6
   134d6:      	xor	a2, t6, a2
   134da:      	xor	a3, a3, a5
   134dc:      	xor	a2, a2, a4
   134de:      	lui	a4, 0xa831c
   134e2:      	addi	a4, a4, 0x66d
   134e6:      	add	a4, a4, t5
   134e8:      	xor	a0, a0, a3
   134ea:      	add	a2, a2, a4
   134ec:      	add	a0, a0, a1
   134ee:      	add	a6, a6, a2
   134f0:      	add	t5, a0, a2
   134f4:      	ld	t6, 0x128(sp)
   134f6:      	add	t6, t6, t3
   134f8:      	xor	a0, s1, t3
   134fc:      	and	a0, a6, a0
   13500:      	xor	t3, a0, t3
   13504:      	srliw	a2, a6, 0x6
   13508:      	slli	a3, a6, 0x1a
   1350c:      	or	s4, a3, a2
   13510:      	srliw	a3, a6, 0xb
   13514:      	slli	a4, a6, 0x15
   13518:      	or	a3, a3, a4
   1351a:      	srliw	a4, a6, 0x19
   1351e:      	slli	s0, a6, 0x7
   13522:      	or	a4, a4, s0
   13524:      	srliw	s0, t5, 0x2
   13528:      	slli	a1, t5, 0x1e
   1352c:      	or	a1, a1, s0
   1352e:      	srliw	s0, t5, 0xd
   13532:      	slli	a5, t5, 0x13
   13536:      	or	a5, a5, s0
   13538:      	srliw	s0, t5, 0x16
   1353c:      	slli	a0, t5, 0xa
   13540:      	or	a0, a0, s0
   13542:      	xor	s0, t1, t2
   13546:      	and	s0, t5, s0
   1354a:      	and	a2, t1, t2
   1354e:      	xor	a2, a2, s0
   13550:      	add	a7, a7, t3
   13552:      	xor	a3, s4, a3
   13556:      	xor	a1, a1, a5
   13558:      	xor	a3, a3, a4
   1355a:      	lui	a4, 0xb0032
   1355e:      	addi	a4, a4, 0x7c8
   13562:      	add	a4, a4, a7
   13564:      	xor	a0, a0, a1
   13566:      	add	a3, a3, a4
   13568:      	add	a0, a0, a2
   1356a:      	add	t0, t0, a3
   1356c:      	add	t3, a0, a3
   13570:      	ld	a7, 0x140(sp)
   13572:      	add	a7, a7, s1
   13574:      	xor	a1, a6, s1
   13578:      	and	a1, t0, a1
   1357c:      	xor	s4, a1, s1
   13580:      	srliw	a2, t0, 0x6
   13584:      	slli	a3, t0, 0x1a
   13588:      	or	a2, a2, a3
   1358a:      	srliw	a3, t0, 0xb
   1358e:      	slli	a5, t0, 0x15
   13592:      	or	a3, a3, a5
   13594:      	srliw	a5, t0, 0x19
   13598:      	slli	a4, t0, 0x7
   1359c:      	or	a4, a4, a5
   1359e:      	srliw	a5, t3, 0x2
   135a2:      	slli	a0, t3, 0x1e
   135a6:      	or	a0, a0, a5
   135a8:      	srliw	a5, t3, 0xd
   135ac:      	slli	s0, t3, 0x13
   135b0:      	or	a5, a5, s0
   135b2:      	srliw	s0, t3, 0x16
   135b6:      	slli	s1, t3, 0xa
   135ba:      	or	s0, s0, s1
   135bc:      	xor	s1, t5, t1
   135c0:      	and	s1, t3, s1
   135c4:      	and	a1, t5, t1
   135c8:      	xor	a1, a1, s1
   135ca:      	add	t6, t6, s4
   135cc:      	xor	a2, a2, a3
   135ce:      	xor	a0, a0, a5
   135d0:      	xor	a2, a2, a4
   135d2:      	lui	a3, 0xbf598
   135d6:      	addi	a3, a3, -0x39
   135da:      	add	a3, a3, t6
   135dc:      	xor	a0, a0, s0
   135de:      	add	a2, a2, a3
   135e0:      	add	a0, a0, a1
   135e2:      	add	t2, t2, a2
   135e4:      	add	t6, a0, a2
   135e8:      	ld	s6, 0x130(sp)
   135ea:      	add	s6, s6, a6
   135ec:      	xor	a0, t0, a6
   135f0:      	and	a0, t2, a0
   135f4:      	xor	a6, a0, a6
   135f8:      	srliw	a1, t2, 0x6
   135fc:      	slli	a3, t2, 0x1a
   13600:      	or	a1, a1, a3
   13602:      	srliw	a3, t2, 0xb
   13606:      	slli	a4, t2, 0x15
   1360a:      	or	a3, a3, a4
   1360c:      	srliw	a4, t2, 0x19
   13610:      	slli	a5, t2, 0x7
   13614:      	or	a4, a4, a5
   13616:      	srliw	a5, t6, 0x2
   1361a:      	slli	a2, t6, 0x1e
   1361e:      	or	a2, a2, a5
   13620:      	srliw	a5, t6, 0xd
   13624:      	slli	s0, t6, 0x13
   13628:      	or	a5, a5, s0
   1362a:      	srliw	s1, t6, 0x16
   1362e:      	slli	s0, t6, 0xa
   13632:      	or	s0, s0, s1
   13634:      	xor	s1, t3, t5
   13638:      	and	s1, t6, s1
   1363c:      	and	a0, t3, t5
   13640:      	xor	a0, a0, s1
   13642:      	add	a6, a6, a7
   13644:      	xor	a1, a1, a3
   13646:      	xor	a2, a2, a5
   13648:      	xor	a1, a1, a4
   1364a:      	lui	a3, 0xc6e01
   1364e:      	addi	a3, a3, -0x40d
   13652:      	add	a3, a3, a6
   13654:      	xor	a2, a2, s0
   13656:      	add	a1, a1, a3
   13658:      	add	a0, a0, a2
   1365a:      	add	a3, a1, t1
   1365e:      	add	t1, a0, a1
   13662:      	ld	a7, 0xd8(sp)
   13664:      	add	a7, a7, t0
   13666:      	xor	a0, t2, t0
   1366a:      	and	a0, a0, a3
   1366c:      	xor	a6, a0, t0
   13670:      	srliw	a1, a3, 0x6
   13674:      	slli	a2, a3, 0x1a
   13678:      	or	t0, a2, a1
   1367c:      	srliw	a2, a3, 0xb
   13680:      	slli	a4, a3, 0x15
   13684:      	or	a2, a2, a4
   13686:      	srliw	a4, a3, 0x19
   1368a:      	slli	a5, a3, 0x7
   1368e:      	or	a4, a4, a5
   13690:      	srliw	a5, t1, 0x2
   13694:      	slli	s1, t1, 0x1e
   13698:      	or	a5, a5, s1
   1369a:      	srliw	s1, t1, 0xd
   1369e:      	slli	s0, t1, 0x13
   136a2:      	or	s0, s0, s1
   136a4:      	srliw	s1, t1, 0x16
   136a8:      	slli	a0, t1, 0xa
   136ac:      	or	a0, a0, s1
   136ae:      	xor	s1, t6, t3
   136b2:      	and	s1, t1, s1
   136b6:      	and	a1, t6, t3
   136ba:      	xor	a1, a1, s1
   136bc:      	add	a6, a6, s6
   136be:      	xor	a2, t0, a2
   136c2:      	xor	a5, a5, s0
   136c4:      	xor	a2, a2, a4
   136c6:      	lui	a4, 0xd5a79
   136ca:      	addi	a4, a4, 0x147
   136ce:      	add	a4, a4, a6
   136d0:      	xor	a0, a0, a5
   136d2:      	add	a2, a2, a4
   136d4:      	add	a0, a0, a1
   136d6:      	add	t5, t5, a2
   136d8:      	add	t0, a0, a2
   136dc:      	ld	a6, 0x138(sp)
   136de:      	add	a6, a6, t2
   136e0:      	xor	a0, a3, t2
   136e4:      	and	a0, t5, a0
   136e8:      	xor	t2, a0, t2
   136ec:      	srliw	a2, t5, 0x6
   136f0:      	slli	a5, t5, 0x1a
   136f4:      	or	s4, a5, a2
   136f8:      	srliw	a5, t5, 0xb
   136fc:      	slli	a4, t5, 0x15
   13700:      	or	a4, a4, a5
   13702:      	srliw	a5, t5, 0x19
   13706:      	slli	s1, t5, 0x7
   1370a:      	or	a5, a5, s1
   1370c:      	srliw	s1, t0, 0x2
   13710:      	slli	a1, t0, 0x1e
   13714:      	or	a1, a1, s1
   13716:      	srliw	s1, t0, 0xd
   1371a:      	slli	s0, t0, 0x13
   1371e:      	or	s0, s0, s1
   13720:      	srliw	s1, t0, 0x16
   13724:      	slli	a0, t0, 0xa
   13728:      	or	a0, a0, s1
   1372a:      	xor	s1, t1, t6
   1372e:      	and	s1, t0, s1
   13732:      	and	a2, t1, t6
   13736:      	xor	a2, a2, s1
   13738:      	add	a7, a7, t2
   1373a:      	xor	a4, s4, a4
   1373e:      	xor	a1, a1, s0
   13740:      	xor	a4, a4, a5
   13742:      	lui	a5, 0x6ca6
   13746:      	addi	a5, a5, 0x351
   1374a:      	add	a5, a5, a7
   1374c:      	xor	a0, a0, a1
   1374e:      	add	a4, a4, a5
   13750:      	add	a0, a0, a2
   13752:      	add	t3, t3, a4
   13754:      	add	t2, a0, a4
   13758:      	ld	a7, 0xd0(sp)
   1375a:      	add	a7, a7, a3
   1375c:      	xor	a1, t5, a3
   13760:      	and	a1, t3, a1
   13764:      	xor	s4, a1, a3
   13768:      	srliw	a2, t3, 0x6
   1376c:      	slli	a3, t3, 0x1a
   13770:      	or	a2, a2, a3
   13772:      	srliw	a3, t3, 0xb
   13776:      	slli	a4, t3, 0x15
   1377a:      	or	a3, a3, a4
   1377c:      	srliw	a4, t3, 0x19
   13780:      	slli	a5, t3, 0x7
   13784:      	or	a4, a4, a5
   13786:      	srliw	a5, t2, 0x2
   1378a:      	slli	a0, t2, 0x1e
   1378e:      	or	a0, a0, a5
   13790:      	srliw	a5, t2, 0xd
   13794:      	slli	s0, t2, 0x13
   13798:      	or	a5, a5, s0
   1379a:      	srliw	s0, t2, 0x16
   1379e:      	slli	s1, t2, 0xa
   137a2:      	or	s0, s0, s1
   137a4:      	xor	s1, t0, t1
   137a8:      	and	s1, t2, s1
   137ac:      	and	a1, t0, t1
   137b0:      	xor	a1, a1, s1
   137b2:      	add	a6, a6, s4
   137b4:      	xor	a2, a2, a3
   137b6:      	xor	a0, a0, a5
   137b8:      	xor	a2, a2, a4
   137ba:      	lui	a3, 0x14293
   137be:      	addi	a3, a3, -0x699
   137c2:      	add	a3, a3, a6
   137c4:      	xor	a0, a0, s0
   137c6:      	add	a3, a3, a2
   137c8:      	add	a0, a0, a1
   137ca:      	add	a2, a3, t6
   137ce:      	add	a6, a0, a3
   137d2:      	ld	s6, 0xc8(sp)
   137d4:      	add	s6, s6, t5
   137d6:      	xor	a0, t3, t5
   137da:      	and	a0, a0, a2
   137dc:      	xor	t5, a0, t5
   137e0:      	srliw	a1, a2, 0x6
   137e4:      	slli	a3, a2, 0x1a
   137e8:      	or	t6, a3, a1
   137ec:      	srliw	a3, a2, 0xb
   137f0:      	slli	a4, a2, 0x15
   137f4:      	or	a3, a3, a4
   137f6:      	srliw	a4, a2, 0x19
   137fa:      	slli	a5, a2, 0x7
   137fe:      	or	a4, a4, a5
   13800:      	srliw	a5, a6, 0x2
   13804:      	slli	s1, a6, 0x1e
   13808:      	or	a5, a5, s1
   1380a:      	srliw	s1, a6, 0xd
   1380e:      	slli	s0, a6, 0x13
   13812:      	or	s0, s0, s1
   13814:      	srliw	s1, a6, 0x16
   13818:      	slli	a0, a6, 0xa
   1381c:      	or	a0, a0, s1
   1381e:      	xor	s1, t2, t0
   13822:      	and	s1, a6, s1
   13826:      	and	a1, t2, t0
   1382a:      	xor	a1, a1, s1
   1382c:      	add	a7, a7, t5
   1382e:      	xor	a3, t6, a3
   13832:      	xor	a5, a5, s0
   13834:      	xor	a3, a3, a4
   13836:      	lui	a4, 0x27b71
   1383a:      	addi	a4, a4, -0x57b
   1383e:      	add	a4, a4, a7
   13840:      	xor	a0, a0, a5
   13842:      	add	a3, a3, a4
   13844:      	add	a0, a0, a1
   13846:      	add	t1, t1, a3
   13848:      	add	t5, a0, a3
   1384c:      	ld	a7, 0x120(sp)
   1384e:      	add	a7, a7, t3
   13850:      	xor	a0, a2, t3
   13854:      	and	a0, t1, a0
   13858:      	xor	t3, a0, t3
   1385c:      	srliw	a1, t1, 0x6
   13860:      	slli	a4, t1, 0x1a
   13864:      	or	t6, a4, a1
   13868:      	srliw	a4, t1, 0xb
   1386c:      	slli	a5, t1, 0x15
   13870:      	or	a4, a4, a5
   13872:      	srliw	a5, t1, 0x19
   13876:      	slli	s1, t1, 0x7
   1387a:      	or	a5, a5, s1
   1387c:      	srliw	s1, t5, 0x2
   13880:      	slli	a3, t5, 0x1e
   13884:      	or	a3, a3, s1
   13886:      	srliw	s1, t5, 0xd
   1388a:      	slli	s0, t5, 0x13
   1388e:      	or	s0, s0, s1
   13890:      	srliw	s1, t5, 0x16
   13894:      	slli	a0, t5, 0xa
   13898:      	or	a0, a0, s1
   1389a:      	xor	s1, a6, t2
   1389e:      	and	s1, t5, s1
   138a2:      	and	a1, a6, t2
   138a6:      	xor	a1, a1, s1
   138a8:      	add	t3, t3, s6
   138aa:      	xor	a4, t6, a4
   138ae:      	xor	a3, a3, s0
   138b0:      	xor	a4, a4, a5
   138b2:      	lui	a5, 0x2e1b2
   138b6:      	addi	a5, a5, 0x138
   138ba:      	add	a5, a5, t3
   138bc:      	xor	a0, a0, a3
   138be:      	add	a5, a5, a4
   138c0:      	add	a0, a0, a1
   138c2:      	add	t0, t0, a5
   138c4:      	add	t3, a0, a5
   138c8:      	ld	s6, 0x1c0(sp)
   138ca:      	add	s6, s6, a2
   138cc:      	xor	a0, t1, a2
   138d0:      	and	a0, t0, a0
   138d4:      	xor	t6, a0, a2
   138d8:      	srliw	a2, t0, 0x6
   138dc:      	slli	a3, t0, 0x1a
   138e0:      	or	a2, a2, a3
   138e2:      	srliw	a3, t0, 0xb
   138e6:      	slli	a5, t0, 0x15
   138ea:      	or	a3, a3, a5
   138ec:      	srliw	a5, t0, 0x19
   138f0:      	slli	a4, t0, 0x7
   138f4:      	or	a4, a4, a5
   138f6:      	srliw	a5, t3, 0x2
   138fa:      	slli	a1, t3, 0x1e
   138fe:      	or	a1, a1, a5
   13900:      	srliw	a5, t3, 0xd
   13904:      	slli	s0, t3, 0x13
   13908:      	or	a5, a5, s0
   1390a:      	srliw	s0, t3, 0x16
   1390e:      	slli	s1, t3, 0xa
   13912:      	or	s0, s0, s1
   13914:      	xor	s1, t5, a6
   13918:      	and	s1, t3, s1
   1391c:      	and	a0, t5, a6
   13920:      	xor	a0, a0, s1
   13922:      	add	a7, a7, t6
   13924:      	xor	a2, a2, a3
   13926:      	xor	a1, a1, a5
   13928:      	xor	a2, a2, a4
   1392a:      	lui	a3, 0x4d2c7
   1392e:      	addi	a3, a3, -0x204
   13932:      	add	a3, a3, a7
   13934:      	xor	a1, a1, s0
   13936:      	add	a2, a2, a3
   13938:      	add	a0, a0, a1
   1393a:      	add	t2, t2, a2
   1393c:      	add	t6, a0, a2
   13940:      	ld	a7, 0xb8(sp)
   13942:      	add	a7, a7, t1
   13944:      	xor	a1, t0, t1
   13948:      	and	a1, t2, a1
   1394c:      	xor	t1, a1, t1
   13950:      	srliw	a2, t2, 0x6
   13954:      	slli	a3, t2, 0x1a
   13958:      	or	a2, a2, a3
   1395a:      	srliw	a3, t2, 0xb
   1395e:      	slli	a4, t2, 0x15
   13962:      	or	a3, a3, a4
   13964:      	srliw	a4, t2, 0x19
   13968:      	slli	a5, t2, 0x7
   1396c:      	or	a4, a4, a5
   1396e:      	srliw	a5, t6, 0x2
   13972:      	slli	a0, t6, 0x1e
   13976:      	or	a0, a0, a5
   13978:      	srliw	a5, t6, 0xd
   1397c:      	slli	s1, t6, 0x13
   13980:      	or	a5, a5, s1
   13982:      	srliw	s1, t6, 0x16
   13986:      	slli	s0, t6, 0xa
   1398a:      	or	s0, s0, s1
   1398c:      	xor	s1, t3, t5
   13990:      	and	s1, t6, s1
   13994:      	and	a1, t3, t5
   13998:      	xor	a1, a1, s1
   1399a:      	add	t1, t1, s6
   1399c:      	xor	a2, a2, a3
   1399e:      	xor	a0, a0, a5
   139a0:      	xor	a2, a2, a4
   139a2:      	lui	a3, 0x53381
   139a6:      	addi	a3, a3, -0x2ed
   139aa:      	add	a3, a3, t1
   139ac:      	xor	a0, a0, s0
   139ae:      	add	a2, a2, a3
   139b0:      	add	a0, a0, a1
   139b2:      	add	a6, a6, a2
   139b4:      	add	t1, a0, a2
   139b8:      	ld	s6, 0x1c8(sp)
   139ba:      	add	s6, s6, t0
   139bc:      	xor	a0, t2, t0
   139c0:      	and	a0, a6, a0
   139c4:      	xor	t0, a0, t0
   139c8:      	srliw	a1, a6, 0x6
   139cc:      	slli	a3, a6, 0x1a
   139d0:      	or	a1, a1, a3
   139d2:      	srliw	a3, a6, 0xb
   139d6:      	slli	a4, a6, 0x15
   139da:      	or	a3, a3, a4
   139dc:      	srliw	a4, a6, 0x19
   139e0:      	slli	a5, a6, 0x7
   139e4:      	or	a4, a4, a5
   139e6:      	srliw	a5, t1, 0x2
   139ea:      	slli	a2, t1, 0x1e
   139ee:      	or	a2, a2, a5
   139f0:      	srliw	a5, t1, 0xd
   139f4:      	slli	s0, t1, 0x13
   139f8:      	or	a5, a5, s0
   139fa:      	srliw	s1, t1, 0x16
   139fe:      	slli	s0, t1, 0xa
   13a02:      	or	s0, s0, s1
   13a04:      	xor	s1, t6, t3
   13a08:      	and	s1, t1, s1
   13a0c:      	and	a0, t6, t3
   13a10:      	xor	a0, a0, s1
   13a12:      	add	a7, a7, t0
   13a14:      	xor	a1, a1, a3
   13a16:      	xor	a2, a2, a5
   13a18:      	xor	a1, a1, a4
   13a1a:      	lui	a3, 0x650a7
   13a1e:      	addi	a3, a3, 0x354
   13a22:      	add	a3, a3, a7
   13a24:      	xor	a2, a2, s0
   13a26:      	add	a1, a1, a3
   13a28:      	add	a0, a0, a2
   13a2a:      	add	t5, t5, a1
   13a2c:      	add	t0, a0, a1
   13a30:      	ld	s4, 0x178(sp)
   13a32:      	add	s4, s4, t2
   13a34:      	xor	a1, a6, t2
   13a38:      	and	a1, t5, a1
   13a3c:      	xor	a7, a1, t2
   13a40:      	srliw	a2, t5, 0x6
   13a44:      	slli	a5, t5, 0x1a
   13a48:      	or	a2, a2, a5
   13a4a:      	srliw	a5, t5, 0xb
   13a4e:      	slli	a4, t5, 0x15
   13a52:      	or	a4, a4, a5
   13a54:      	srliw	a5, t5, 0x19
   13a58:      	slli	s1, t5, 0x7
   13a5c:      	or	a5, a5, s1
   13a5e:      	srliw	s1, t0, 0x2
   13a62:      	slli	a3, t0, 0x1e
   13a66:      	or	a3, a3, s1
   13a68:      	srliw	s1, t0, 0xd
   13a6c:      	slli	s0, t0, 0x13
   13a70:      	or	s0, s0, s1
   13a72:      	srliw	s1, t0, 0x16
   13a76:      	slli	a0, t0, 0xa
   13a7a:      	or	a0, a0, s1
   13a7c:      	xor	s1, t1, t6
   13a80:      	and	s1, t0, s1
   13a84:      	and	a1, t1, t6
   13a88:      	xor	a1, a1, s1
   13a8a:      	add	a7, a7, s6
   13a8c:      	xor	a2, a2, a4
   13a8e:      	xor	a3, a3, s0
   13a90:      	xor	a2, a2, a5
   13a92:      	lui	a4, 0x766a1
   13a96:      	addi	a4, a4, -0x545
   13a9a:      	add	a4, a4, a7
   13a9c:      	xor	a0, a0, a3
   13a9e:      	add	a2, a2, a4
   13aa0:      	add	a0, a0, a1
   13aa2:      	add	t3, t3, a2
   13aa4:      	add	t2, a0, a2
   13aa8:      	ld	a7, 0x1b8(sp)
   13aaa:      	add	a7, a7, a6
   13aac:      	xor	a0, t5, a6
   13ab0:      	and	a0, t3, a0
   13ab4:      	xor	a6, a0, a6
   13ab8:      	srliw	a2, t3, 0x6
   13abc:      	slli	a3, t3, 0x1a
   13ac0:      	or	a2, a2, a3
   13ac2:      	srliw	a3, t3, 0xb
   13ac6:      	slli	a4, t3, 0x15
   13aca:      	or	a3, a3, a4
   13acc:      	srliw	a4, t3, 0x19
   13ad0:      	slli	a5, t3, 0x7
   13ad4:      	or	a4, a4, a5
   13ad6:      	srliw	a5, t2, 0x2
   13ada:      	slli	a1, t2, 0x1e
   13ade:      	or	a1, a1, a5
   13ae0:      	srliw	a5, t2, 0xd
   13ae4:      	slli	s0, t2, 0x13
   13ae8:      	or	a5, a5, s0
   13aea:      	srliw	s1, t2, 0x16
   13aee:      	slli	s0, t2, 0xa
   13af2:      	or	s0, s0, s1
   13af4:      	xor	s1, t0, t1
   13af8:      	and	s1, t2, s1
   13afc:      	and	a0, t0, t1
   13b00:      	xor	a0, a0, s1
   13b02:      	add	a6, a6, s4
   13b04:      	xor	a2, a2, a3
   13b06:      	xor	a1, a1, a5
   13b08:      	xor	a2, a2, a4
   13b0a:      	lui	a3, 0x81c2d
   13b0e:      	addi	a3, a3, -0x6d2
   13b12:      	add	a3, a3, a6
   13b14:      	xor	a1, a1, s0
   13b16:      	add	a2, a2, a3
   13b18:      	add	a1, a1, a0
   13b1a:      	add	a0, a2, t6
   13b1e:      	add	t6, a1, a2
   13b22:      	ld	a6, 0xb0(sp)
   13b24:      	add	a6, a6, t5
   13b26:      	xor	a1, t3, t5
   13b2a:      	and	a1, a1, a0
   13b2c:      	xor	t5, a1, t5
   13b30:      	srliw	a2, a0, 0x6
   13b34:      	slli	a3, a0, 0x1a
   13b38:      	or	s4, a3, a2
   13b3c:      	srliw	a3, a0, 0xb
   13b40:      	slli	a4, a0, 0x15
   13b44:      	or	a3, a3, a4
   13b46:      	srliw	a4, a0, 0x19
   13b4a:      	slli	a5, a0, 0x7
   13b4e:      	or	a4, a4, a5
   13b50:      	srliw	a5, t6, 0x2
   13b54:      	slli	s1, t6, 0x1e
   13b58:      	or	a5, a5, s1
   13b5a:      	srliw	s1, t6, 0xd
   13b5e:      	slli	s0, t6, 0x13
   13b62:      	or	s0, s0, s1
   13b64:      	srliw	s1, t6, 0x16
   13b68:      	slli	a1, t6, 0xa
   13b6c:      	or	a1, a1, s1
   13b6e:      	xor	s1, t2, t0
   13b72:      	and	s1, t6, s1
   13b76:      	and	a2, t2, t0
   13b7a:      	xor	a2, a2, s1
   13b7c:      	add	a7, a7, t5
   13b7e:      	xor	a3, s4, a3
   13b82:      	xor	a5, a5, s0
   13b84:      	xor	a3, a3, a4
   13b86:      	lui	a4, 0x92723
   13b8a:      	addi	a4, a4, -0x37b
   13b8e:      	add	a4, a4, a7
   13b90:      	xor	a1, a1, a5
   13b92:      	add	a3, a3, a4
   13b94:      	add	a1, a1, a2
   13b96:      	add	t1, t1, a3
   13b98:      	add	t5, a1, a3
   13b9c:      	ld	a7, 0x98(sp)
   13b9e:      	add	a7, a7, t3
   13ba0:      	xor	a1, a0, t3
   13ba4:      	and	a1, t1, a1
   13ba8:      	xor	t3, a1, t3
   13bac:      	srliw	a3, t1, 0x6
   13bb0:      	slli	a5, t1, 0x1a
   13bb4:      	or	s4, a5, a3
   13bb8:      	srliw	a5, t1, 0xb
   13bbc:      	slli	a4, t1, 0x15
   13bc0:      	or	a4, a4, a5
   13bc2:      	srliw	a5, t1, 0x19
   13bc6:      	slli	s1, t1, 0x7
   13bca:      	or	a5, a5, s1
   13bcc:      	srliw	s1, t5, 0x2
   13bd0:      	slli	a2, t5, 0x1e
   13bd4:      	or	a2, a2, s1
   13bd6:      	srliw	s1, t5, 0xd
   13bda:      	slli	s0, t5, 0x13
   13bde:      	or	s0, s0, s1
   13be0:      	srliw	s1, t5, 0x16
   13be4:      	slli	a1, t5, 0xa
   13be8:      	or	a1, a1, s1
   13bea:      	xor	s1, t6, t2
   13bee:      	and	s1, t5, s1
   13bf2:      	and	a3, t6, t2
   13bf6:      	xor	a3, a3, s1
   13bf8:      	add	a6, a6, t3
   13bfa:      	xor	a4, s4, a4
   13bfe:      	xor	a2, a2, s0
   13c00:      	xor	a4, a4, a5
   13c02:      	lui	a5, 0xa2bff
   13c06:      	addi	a5, a5, -0x75f
   13c0a:      	add	a5, a5, a6
   13c0c:      	xor	a1, a1, a2
   13c0e:      	add	a4, a4, a5
   13c10:      	add	a1, a1, a3
   13c12:      	add	t0, t0, a4
   13c14:      	add	t3, a1, a4
   13c18:      	ld	s2, 0xa8(sp)
   13c1a:      	add	s2, s2, a0
   13c1c:      	xor	a1, t1, a0
   13c20:      	and	a1, t0, a1
   13c24:      	xor	a6, a1, a0
   13c28:      	srliw	a1, t0, 0x6
   13c2c:      	slli	a2, t0, 0x1a
   13c30:      	or	a1, a1, a2
   13c32:      	srliw	a2, t0, 0xb
   13c36:      	slli	a4, t0, 0x15
   13c3a:      	or	a2, a2, a4
   13c3c:      	srliw	a4, t0, 0x19
   13c40:      	slli	a5, t0, 0x7
   13c44:      	or	a4, a4, a5
   13c46:      	srliw	a5, t3, 0x2
   13c4a:      	slli	a3, t3, 0x1e
   13c4e:      	or	a3, a3, a5
   13c50:      	srliw	a5, t3, 0xd
   13c54:      	slli	s0, t3, 0x13
   13c58:      	or	a5, a5, s0
   13c5a:      	srliw	s1, t3, 0x16
   13c5e:      	slli	s0, t3, 0xa
   13c62:      	or	s0, s0, s1
   13c64:      	xor	s1, t5, t6
   13c68:      	and	s1, t3, s1
   13c6c:      	and	a0, t5, t6
   13c70:      	xor	a0, a0, s1
   13c72:      	add	a6, a6, a7
   13c74:      	xor	a1, a1, a2
   13c76:      	xor	a3, a3, a5
   13c78:      	xor	a1, a1, a4
   13c7a:      	lui	a2, 0xa81a6
   13c7e:      	addi	a2, a2, 0x64b
   13c82:      	add	a2, a2, a6
   13c84:      	xor	a3, a3, s0
   13c86:      	add	a2, a2, a1
   13c88:      	add	a0, a0, a3
   13c8a:      	add	a1, a2, t2
   13c8e:      	add	a6, a0, a2
   13c92:      	ld	s5, 0xa0(sp)
   13c94:      	add	s5, s5, t1
   13c96:      	xor	a0, t0, t1
   13c9a:      	and	a0, a0, a1
   13c9c:      	xor	a7, a0, t1
   13ca0:      	srliw	a2, a1, 0x6
   13ca4:      	slli	a3, a1, 0x1a
   13ca8:      	or	t1, a3, a2
   13cac:      	srliw	a3, a1, 0xb
   13cb0:      	slli	a4, a1, 0x15
   13cb4:      	or	a3, a3, a4
   13cb6:      	srliw	a4, a1, 0x19
   13cba:      	slli	a5, a1, 0x7
   13cbe:      	or	a4, a4, a5
   13cc0:      	srliw	a5, a6, 0x2
   13cc4:      	slli	s1, a6, 0x1e
   13cc8:      	or	a5, a5, s1
   13cca:      	srliw	s1, a6, 0xd
   13cce:      	slli	s0, a6, 0x13
   13cd2:      	or	s0, s0, s1
   13cd4:      	srliw	s1, a6, 0x16
   13cd8:      	slli	a0, a6, 0xa
   13cdc:      	or	a0, a0, s1
   13cde:      	xor	s1, t3, t5
   13ce2:      	and	s1, a6, s1
   13ce6:      	and	a2, t3, t5
   13cea:      	xor	a2, a2, s1
   13cec:      	add	a7, a7, s2
   13cee:      	xor	a3, t1, a3
   13cf2:      	xor	a5, a5, s0
   13cf4:      	xor	a3, a3, a4
   13cf6:      	lui	a4, 0xc24b9
   13cfa:      	addi	a4, a4, -0x490
   13cfe:      	add	a4, a4, a7
   13d00:      	xor	a0, a0, a5
   13d02:      	add	a3, a3, a4
   13d04:      	add	a0, a0, a2
   13d06:      	add	s0, a3, t6
   13d0a:      	add	a7, a0, a3
   13d0e:      	ld	t2, 0x148(sp)
   13d10:      	add	t2, t2, t0
   13d12:      	xor	a2, a1, t0
   13d16:      	and	a2, a2, s0
   13d18:      	xor	t0, a2, t0
   13d1c:      	srliw	a3, s0, 0x6
   13d20:      	slli	a4, s0, 0x1a
   13d24:      	or	t1, a4, a3
   13d28:      	srliw	a4, s0, 0xb
   13d2c:      	slli	a5, s0, 0x15
   13d30:      	or	t6, a5, a4
   13d34:      	srliw	a5, s0, 0x19
   13d38:      	slli	s1, s0, 0x7
   13d3c:      	or	a5, a5, s1
   13d3e:      	srliw	s1, a7, 0x2
   13d42:      	slli	a0, a7, 0x1e
   13d46:      	or	a0, a0, s1
   13d48:      	srliw	s1, a7, 0xd
   13d4c:      	slli	a2, a7, 0x13
   13d50:      	or	a2, a2, s1
   13d52:      	srliw	s1, a7, 0x16
   13d56:      	slli	a3, a7, 0xa
   13d5a:      	or	a3, a3, s1
   13d5c:      	xor	s1, a6, t3
   13d60:      	and	s1, a7, s1
   13d64:      	and	a4, a6, t3
   13d68:      	xor	a4, a4, s1
   13d6a:      	add	t0, t0, s5
   13d6c:      	xor	s1, t1, t6
   13d70:      	xor	a0, a0, a2
   13d72:      	xor	a5, a5, s1
   13d74:      	lui	a2, 0xc76c5
   13d78:      	addi	a2, a2, 0x1a3
   13d7c:      	add	a2, a2, t0
   13d7e:      	xor	a0, a0, a3
   13d80:      	add	a5, a5, a2
   13d82:      	add	a0, a0, a4
   13d84:      	add	a2, a5, t5
   13d88:      	add	t0, a0, a5
   13d8c:      	ld	t6, 0x1d0(sp)
   13d8e:      	add	t6, t6, a1
   13d90:      	xor	a0, s0, a1
   13d94:      	and	a0, a0, a2
   13d96:      	xor	t1, a0, a1
   13d9a:      	srliw	a1, a2, 0x6
   13d9e:      	slli	a3, a2, 0x1a
   13da2:      	or	t5, a3, a1
   13da6:      	srliw	a3, a2, 0xb
   13daa:      	slli	a5, a2, 0x15
   13dae:      	or	s2, a5, a3
   13db2:      	srliw	a5, a2, 0x19
   13db6:      	slli	a4, a2, 0x7
   13dba:      	or	a4, a4, a5
   13dbc:      	srliw	a5, t0, 0x2
   13dc0:      	slli	s1, t0, 0x1e
   13dc4:      	or	a5, a5, s1
   13dc6:      	srliw	s1, t0, 0xd
   13dca:      	slli	a0, t0, 0x13
   13dce:      	or	a0, a0, s1
   13dd0:      	srliw	s1, t0, 0x16
   13dd4:      	slli	a1, t0, 0xa
   13dd8:      	or	a1, a1, s1
   13dda:      	xor	s1, a7, a6
   13dde:      	and	s1, t0, s1
   13de2:      	and	a3, a7, a6
   13de6:      	xor	a3, a3, s1
   13de8:      	add	t1, t1, t2
   13dea:      	xor	s1, t5, s2
   13dee:      	xor	a0, a0, a5
   13df0:      	xor	a4, a4, s1
   13df2:      	lui	a5, 0xd192f
   13df6:      	addi	a5, a5, -0x7e7
   13dfa:      	add	a5, a5, t1
   13dfc:      	xor	a0, a0, a1
   13dfe:      	add	a4, a4, a5
   13e00:      	add	a0, a0, a3
   13e02:      	add	s1, a4, t3
   13e06:      	add	t2, a0, a4
   13e0a:      	add	t3, s7, s0
   13e0e:      	xor	a0, a2, s0
   13e12:      	and	a0, a0, s1
   13e14:      	xor	t1, a0, s0
   13e18:      	srliw	a1, s1, 0x6
   13e1c:      	slli	a4, s1, 0x1a
   13e20:      	or	t5, a4, a1
   13e24:      	srliw	a4, s1, 0xb
   13e28:      	slli	a5, s1, 0x15
   13e2c:      	or	s2, a5, a4
   13e30:      	srliw	a5, s1, 0x19
   13e34:      	slli	a3, s1, 0x7
   13e38:      	or	a3, a3, a5
   13e3a:      	srliw	a5, t2, 0x2
   13e3e:      	slli	s0, t2, 0x1e
   13e42:      	or	a5, a5, s0
   13e44:      	srliw	s0, t2, 0xd
   13e48:      	slli	a0, t2, 0x13
   13e4c:      	or	a0, a0, s0
   13e4e:      	srliw	s0, t2, 0x16
   13e52:      	slli	a1, t2, 0xa
   13e56:      	or	a1, a1, s0
   13e58:      	xor	s0, t0, a7
   13e5c:      	and	s0, t2, s0
   13e60:      	and	a4, t0, a7
   13e64:      	xor	a4, a4, s0
   13e66:      	add	t1, t1, t6
   13e68:      	xor	s0, t5, s2
   13e6c:      	xor	a0, a0, a5
   13e6e:      	xor	a3, a3, s0
   13e70:      	lui	a5, 0xd6990
   13e74:      	addi	a5, a5, 0x624
   13e78:      	add	a5, a5, t1
   13e7a:      	xor	a0, a0, a1
   13e7c:      	add	a1, a5, a3
   13e80:      	add	a0, a0, a4
   13e82:      	add	a6, a6, a1
   13e84:      	add	t6, a0, a1
   13e88:      	add	t5, t4, a2
   13e8c:      	xor	a0, s1, a2
   13e90:      	and	a0, a6, a0
   13e94:      	xor	t1, a0, a2
   13e98:      	srliw	a2, a6, 0x6
   13e9c:      	slli	a3, a6, 0x1a
   13ea0:      	or	s2, a3, a2
   13ea4:      	srliw	a3, a6, 0xb
   13ea8:      	slli	a4, a6, 0x15
   13eac:      	or	a3, a3, a4
   13eae:      	srliw	a4, a6, 0x19
   13eb2:      	slli	a5, a6, 0x7
   13eb6:      	or	a4, a4, a5
   13eb8:      	srliw	a5, t6, 0x2
   13ebc:      	slli	a1, t6, 0x1e
   13ec0:      	or	a1, a1, a5
   13ec2:      	srliw	a5, t6, 0xd
   13ec6:      	slli	s0, t6, 0x13
   13eca:      	or	a5, a5, s0
   13ecc:      	srliw	s0, t6, 0x16
   13ed0:      	slli	a0, t6, 0xa
   13ed4:      	or	a0, a0, s0
   13ed6:      	xor	s0, t2, t0
   13eda:      	and	s0, t6, s0
   13ede:      	and	a2, t2, t0
   13ee2:      	xor	a2, a2, s0
   13ee4:      	add	t1, t1, t3
   13ee6:      	xor	a3, s2, a3
   13eea:      	xor	a1, a1, a5
   13eec:      	xor	a3, a3, a4
   13eee:      	lui	a4, 0xf40e3
   13ef2:      	addi	a4, a4, 0x585
   13ef6:      	add	a4, a4, t1
   13ef8:      	xor	a0, a0, a1
   13efa:      	add	a3, a3, a4
   13efc:      	add	a2, a2, a0
   13efe:      	add	a7, a7, a3
   13f00:      	add	t3, a2, a3
   13f04:      	ld	t1, 0x58(sp)
   13f06:      	add	t1, t1, s1
   13f08:      	xor	a1, a6, s1
   13f0c:      	and	a1, a7, a1
   13f10:      	xor	s2, a1, s1
   13f14:      	srliw	a2, a7, 0x6
   13f18:      	slli	a3, a7, 0x1a
   13f1c:      	or	a2, a2, a3
   13f1e:      	srliw	a3, a7, 0xb
   13f22:      	slli	a4, a7, 0x15
   13f26:      	or	a3, a3, a4
   13f28:      	srliw	a4, a7, 0x19
   13f2c:      	slli	a0, a7, 0x7
   13f30:      	or	a0, a0, a4
   13f32:      	srliw	a4, t3, 0x2
   13f36:      	slli	a5, t3, 0x1e
   13f3a:      	or	a4, a4, a5
   13f3c:      	srliw	a5, t3, 0xd
   13f40:      	slli	s0, t3, 0x13
   13f44:      	or	a5, a5, s0
   13f46:      	srliw	s0, t3, 0x16
   13f4a:      	slli	s1, t3, 0xa
   13f4e:      	or	s0, s0, s1
   13f50:      	xor	s1, t6, t2
   13f54:      	and	s1, t3, s1
   13f58:      	and	a1, t6, t2
   13f5c:      	xor	a1, a1, s1
   13f5e:      	add	t5, t5, s2
   13f60:      	xor	a2, a2, a3
   13f62:      	xor	a4, a4, a5
   13f64:      	xor	a0, a0, a2
   13f66:      	lui	a2, 0x106aa
   13f6a:      	addi	a2, a2, 0x70
   13f6e:      	add	a2, a2, t5
   13f70:      	xor	a4, a4, s0
   13f72:      	add	a0, a0, a2
   13f74:      	add	a1, a1, a4
   13f76:      	add	t0, t0, a0
   13f78:      	add	t5, a1, a0
   13f7c:      	ld	ra, 0x50(sp)
   13f7e:      	add	ra, ra, a6
   13f80:      	xor	a0, a7, a6
   13f84:      	and	a0, t0, a0
   13f88:      	xor	a6, a0, a6
   13f8c:      	srliw	a1, t0, 0x6
   13f90:      	slli	a3, t0, 0x1a
   13f94:      	or	a1, a1, a3
   13f96:      	srliw	a3, t0, 0xb
   13f9a:      	slli	a5, t0, 0x15
   13f9e:      	or	a3, a3, a5
   13fa0:      	srliw	a5, t0, 0x19
   13fa4:      	slli	a4, t0, 0x7
   13fa8:      	or	a4, a4, a5
   13faa:      	srliw	a5, t5, 0x2
   13fae:      	slli	a2, t5, 0x1e
   13fb2:      	or	a2, a2, a5
   13fb4:      	srliw	a5, t5, 0xd
   13fb8:      	slli	s0, t5, 0x13
   13fbc:      	or	a5, a5, s0
   13fbe:      	srliw	s0, t5, 0x16
   13fc2:      	slli	s1, t5, 0xa
   13fc6:      	or	s0, s0, s1
   13fc8:      	xor	s1, t3, t6
   13fcc:      	and	s1, t5, s1
   13fd0:      	and	a0, t3, t6
   13fd4:      	xor	a0, a0, s1
   13fd6:      	add	a6, a6, t1
   13fd8:      	xor	a1, a1, a3
   13fda:      	xor	a2, a2, a5
   13fdc:      	xor	a1, a1, a4
   13fde:      	lui	a3, 0x19a4c
   13fe2:      	addi	a3, a3, 0x116
   13fe6:      	add	a3, a3, a6
   13fe8:      	xor	a2, a2, s0
   13fea:      	add	a1, a1, a3
   13fec:      	add	a0, a0, a2
   13fee:      	add	t2, t2, a1
   13ff0:      	add	t1, a0, a1
   13ff4:      	ld	t4, 0x70(sp)
   13ff6:      	add	t4, t4, a7
   13ff8:      	xor	a0, t0, a7
   13ffc:      	and	a0, t2, a0
   14000:      	xor	a6, a0, a7
   14004:      	srliw	a1, t2, 0x6
   14008:      	slli	a2, t2, 0x1a
   1400c:      	or	a1, a1, a2
   1400e:      	srliw	a2, t2, 0xb
   14012:      	slli	a4, t2, 0x15
   14016:      	or	a2, a2, a4
   14018:      	srliw	a4, t2, 0x19
   1401c:      	slli	a5, t2, 0x7
   14020:      	or	a4, a4, a5
   14022:      	srliw	a5, t1, 0x2
   14026:      	slli	a3, t1, 0x1e
   1402a:      	or	a3, a3, a5
   1402c:      	srliw	a5, t1, 0xd
   14030:      	slli	s1, t1, 0x13
   14034:      	or	a5, a5, s1
   14036:      	srliw	s1, t1, 0x16
   1403a:      	slli	s0, t1, 0xa
   1403e:      	or	s0, s0, s1
   14040:      	xor	s1, t5, t3
   14044:      	and	s1, t1, s1
   14048:      	and	a0, t5, t3
   1404c:      	xor	a0, a0, s1
   1404e:      	add	a6, a6, ra
   14050:      	xor	a1, a1, a2
   14052:      	xor	a3, a3, a5
   14054:      	xor	a1, a1, a4
   14056:      	lui	a2, 0x1e377
   1405a:      	addi	a2, a2, -0x3f8
   1405e:      	add	a2, a2, a6
   14060:      	xor	a3, a3, s0
   14062:      	add	a2, a2, a1
   14064:      	add	a0, a0, a3
   14066:      	add	t6, t6, a2
   14068:      	add	s2, a0, a2
   1406c:      	ld	s7, 0x180(sp)
   1406e:      	add	s7, s7, t0
   14070:      	xor	a2, t2, t0
   14074:      	and	a2, t6, a2
   14078:      	xor	a6, a2, t0
   1407c:      	srliw	a3, t6, 0x6
   14080:      	slli	a4, t6, 0x1a
   14084:      	or	a3, a3, a4
   14086:      	srliw	a4, t6, 0xb
   1408a:      	slli	a5, t6, 0x15
   1408e:      	or	a4, a4, a5
   14090:      	srliw	a5, t6, 0x19
   14094:      	slli	a1, t6, 0x7
   14098:      	or	a1, a1, a5
   1409a:      	srliw	a5, s2, 0x2
   1409e:      	slli	a0, s2, 0x1e
   140a2:      	or	a0, a0, a5
   140a4:      	srliw	a5, s2, 0xd
   140a8:      	slli	s0, s2, 0x13
   140ac:      	or	a5, a5, s0
   140ae:      	srliw	s0, s2, 0x16
   140b2:      	slli	s1, s2, 0xa
   140b6:      	or	s0, s0, s1
   140b8:      	xor	s1, t1, t5
   140bc:      	and	s1, s2, s1
   140c0:      	and	a2, t1, t5
   140c4:      	xor	a2, a2, s1
   140c6:      	add	a6, a6, t4
   140c8:      	xor	a3, a3, a4
   140ca:      	xor	a0, a0, a5
   140cc:      	xor	a1, a1, a3
   140ce:      	lui	a3, 0x27487
   140d2:      	addi	a3, a3, 0x74c
   140d6:      	add	a3, a3, a6
   140d8:      	xor	a0, a0, s0
   140da:      	add	a1, a1, a3
   140dc:      	add	a0, a0, a2
   140de:      	add	a5, a1, t3
   140e2:      	add	a7, a0, a1
   140e6:      	add	t0, s8, t2
   140ea:      	xor	a0, t6, t2
   140ee:      	and	a0, a0, a5
   140f0:      	xor	a6, a0, t2
   140f4:      	srliw	a1, a5, 0x6
   140f8:      	slli	a2, a5, 0x1a
   140fc:      	or	t2, a2, a1
   14100:      	srliw	a2, a5, 0xb
   14104:      	slli	a3, a5, 0x15
   14108:      	or	a2, a2, a3
   1410a:      	srliw	a3, a5, 0x19
   1410e:      	slli	a4, a5, 0x7
   14112:      	or	a3, a3, a4
   14114:      	srliw	a4, a7, 0x2
   14118:      	slli	s1, a7, 0x1e
   1411c:      	or	a4, a4, s1
   1411e:      	srliw	s1, a7, 0xd
   14122:      	slli	s0, a7, 0x13
   14126:      	or	s0, s0, s1
   14128:      	srliw	s1, a7, 0x16
   1412c:      	slli	a0, a7, 0xa
   14130:      	or	a0, a0, s1
   14132:      	xor	s1, s2, t1
   14136:      	and	s1, a7, s1
   1413a:      	and	a1, s2, t1
   1413e:      	xor	a1, a1, s1
   14140:      	add	a6, a6, s7
   14142:      	xor	a2, t2, a2
   14146:      	xor	a4, a4, s0
   14148:      	xor	a2, a2, a3
   1414a:      	lui	a3, 0x34b0c
   1414e:      	addi	a3, a3, -0x34b
   14152:      	add	a3, a3, a6
   14154:      	xor	a0, a0, a4
   14156:      	add	a3, a3, a2
   14158:      	add	a0, a0, a1
   1415a:      	add	t5, t5, a3
   1415c:      	add	a6, a0, a3
   14160:      	ld	s8, 0x170(sp)
   14162:      	add	s8, s8, t6
   14164:      	xor	a0, a5, t6
   14168:      	and	a0, t5, a0
   1416c:      	xor	t2, a0, t6
   14170:      	srliw	a1, t5, 0x6
   14174:      	slli	a3, t5, 0x1a
   14178:      	or	t3, a3, a1
   1417c:      	srliw	a3, t5, 0xb
   14180:      	slli	a4, t5, 0x15
   14184:      	or	a3, a3, a4
   14186:      	srliw	a4, t5, 0x19
   1418a:      	slli	a2, t5, 0x7
   1418e:      	or	a2, a2, a4
   14190:      	srliw	a4, a6, 0x2
   14194:      	slli	s1, a6, 0x1e
   14198:      	or	a4, a4, s1
   1419a:      	srliw	s1, a6, 0xd
   1419e:      	slli	s0, a6, 0x13
   141a2:      	or	s0, s0, s1
   141a4:      	srliw	s1, a6, 0x16
   141a8:      	slli	a0, a6, 0xa
   141ac:      	or	a0, a0, s1
   141ae:      	xor	s1, a7, s2
   141b2:      	and	s1, a6, s1
   141b6:      	and	a1, a7, s2
   141ba:      	xor	a1, a1, s1
   141bc:      	add	t0, t0, t2
   141be:      	xor	a3, t3, a3
   141c2:      	xor	a4, a4, s0
   141c4:      	xor	a2, a2, a3
   141c6:      	lui	a3, 0x391c1
   141ca:      	addi	a3, a3, -0x34d
   141ce:      	add	a3, a3, t0
   141d0:      	xor	a0, a0, a4
   141d2:      	add	a2, a2, a3
   141d4:      	add	a0, a0, a1
   141d6:      	ld	t2, 0x78(sp)
   141d8:      	add	t2, t2, a5
   141da:      	add	t1, t1, a2
   141dc:      	add	t0, a0, a2
   141e0:      	xor	a0, t5, a5
   141e4:      	and	a0, t1, a0
   141e8:      	xor	t3, a0, a5
   141ec:      	srliw	a2, t1, 0x6
   141f0:      	slli	a5, t1, 0x1a
   141f4:      	or	a2, a2, a5
   141f6:      	srliw	a5, t1, 0xb
   141fa:      	slli	a3, t1, 0x15
   141fe:      	or	a3, a3, a5
   14200:      	srliw	a5, t1, 0x19
   14204:      	slli	s1, t1, 0x7
   14208:      	or	a5, a5, s1
   1420a:      	srliw	s1, t0, 0x2
   1420e:      	slli	a4, t0, 0x1e
   14212:      	or	a4, a4, s1
   14214:      	srliw	s1, t0, 0xd
   14218:      	slli	s0, t0, 0x13
   1421c:      	or	s0, s0, s1
   1421e:      	srliw	s1, t0, 0x16
   14222:      	slli	a1, t0, 0xa
   14226:      	or	a1, a1, s1
   14228:      	xor	s1, a6, a7
   1422c:      	and	s1, t0, s1
   14230:      	and	a0, a6, a7
   14234:      	xor	a0, a0, s1
   14236:      	add	t3, t3, s8
   14238:      	xor	a2, a2, a3
   1423a:      	xor	a4, a4, s0
   1423c:      	xor	a2, a2, a5
   1423e:      	lui	a3, 0x4ed8b
   14242:      	addi	a3, a3, -0x5b6
   14246:      	add	a3, a3, t3
   14248:      	xor	a1, a1, a4
   1424a:      	add	a2, a2, a3
   1424c:      	ld	s11, 0x188(sp)
   1424e:      	add	s11, s11, t5
   14250:      	add	a1, a1, a0
   14252:      	xor	a3, t1, t5
   14256:      	add	a0, a2, s2
   1425a:      	add	t3, a1, a2
   1425e:      	and	a3, a3, a0
   14260:      	srliw	a1, a0, 0x6
   14264:      	xor	t4, a3, t5
   14268:      	slli	a3, a0, 0x1a
   1426c:      	or	t5, a3, a1
   14270:      	srliw	a3, a0, 0xb
   14274:      	slli	a4, a0, 0x15
   14278:      	or	a3, a3, a4
   1427a:      	srliw	a4, a0, 0x19
   1427e:      	slli	a5, a0, 0x7
   14282:      	or	a4, a4, a5
   14284:      	srliw	a5, t3, 0x2
   14288:      	slli	s0, t3, 0x1e
   1428c:      	or	a5, a5, s0
   1428e:      	srliw	s0, t3, 0xd
   14292:      	slli	s1, t3, 0x13
   14296:      	or	s0, s0, s1
   14298:      	srliw	s1, t3, 0x16
   1429c:      	slli	a2, t3, 0xa
   142a0:      	or	a2, a2, s1
   142a2:      	xor	s1, t0, a6
   142a6:      	and	s1, t3, s1
   142aa:      	and	a1, t0, a6
   142ae:      	xor	a1, a1, s1
   142b0:      	add	t2, t2, t4
   142b2:      	xor	a3, t5, a3
   142b6:      	xor	a5, a5, s0
   142b8:      	xor	a3, a3, a4
   142ba:      	lui	a4, 0x5b9cd
   142be:      	addi	a4, a4, -0x5b1
   142c2:      	add	a4, a4, t2
   142c4:      	xor	a2, a2, a5
   142c6:      	add	s10, s10, t1
   142c8:      	add	a3, a3, a4
   142ca:      	xor	s1, a0, t1
   142ce:      	add	a2, a2, a1
   142d0:      	add	a4, a3, a7
   142d4:      	add	t2, a2, a3
   142d8:      	and	s1, s1, a4
   142da:      	srliw	a2, a4, 0x6
   142de:      	slli	a3, a4, 0x1a
   142e2:      	xor	a7, s1, t1
   142e6:      	srliw	s1, a4, 0xb
   142ea:      	or	t1, a3, a2
   142ee:      	slli	a3, a4, 0x15
   142f2:      	or	t4, a3, s1
   142f6:      	srliw	s1, a4, 0x19
   142fa:      	slli	a1, a4, 0x7
   142fe:      	or	a1, a1, s1
   14300:      	srliw	s1, t2, 0x2
   14304:      	slli	a5, t2, 0x1e
   14308:      	or	a5, a5, s1
   1430a:      	srliw	s1, t2, 0xd
   1430e:      	slli	s0, t2, 0x13
   14312:      	or	s0, s0, s1
   14314:      	srliw	s1, t2, 0x16
   14318:      	slli	a2, t2, 0xa
   1431c:      	or	a2, a2, s1
   1431e:      	xor	s1, t3, t0
   14322:      	and	s1, t2, s1
   14326:      	and	a3, t3, t0
   1432a:      	xor	a3, a3, s1
   1432c:      	add	a7, a7, s11
   1432e:      	xor	s1, t1, t4
   14332:      	xor	a5, a5, s0
   14334:      	xor	a1, a1, s1
   14336:      	lui	s1, 0x682e7
   1433a:      	addi	s1, s1, -0xd
   1433c:      	add	s1, s1, a7
   1433e:      	ld	a7, 0x80(sp)
   14340:      	add	a7, a7, a0
   14342:      	xor	a2, a2, a5
   14344:      	xor	a5, a4, a0
   14348:      	add	a1, a1, s1
   1434a:      	add	a3, a3, a2
   1434c:      	add	a6, a6, a1
   1434e:      	add	t4, a3, a1
   14352:      	and	a5, a6, a5
   14356:      	srliw	a1, a6, 0x6
   1435a:      	slli	s1, a6, 0x1a
   1435e:      	srliw	s0, a6, 0xb
   14362:      	xor	t1, a5, a0
   14366:      	slli	a5, a6, 0x15
   1436a:      	or	t5, s1, a1
   1436e:      	srliw	s1, a6, 0x19
   14372:      	or	a5, a5, s0
   14374:      	slli	a2, a6, 0x7
   14378:      	or	a2, a2, s1
   1437a:      	srliw	s1, t4, 0x2
   1437e:      	slli	a3, t4, 0x1e
   14382:      	or	a3, a3, s1
   14384:      	srliw	s1, t4, 0xd
   14388:      	slli	s0, t4, 0x13
   1438c:      	or	s0, s0, s1
   1438e:      	srliw	s1, t4, 0x16
   14392:      	slli	a0, t4, 0xa
   14396:      	or	a0, a0, s1
   14398:      	xor	s1, t2, t3
   1439c:      	and	s1, t4, s1
   143a0:      	and	a1, t2, t3
   143a4:      	xor	a1, a1, s1
   143a6:      	add	t1, t1, s10
   143a8:      	xor	a5, t5, a5
   143ac:      	xor	a3, a3, s0
   143ae:      	xor	a2, a2, a5
   143b0:      	lui	a5, 0x748f8
   143b4:      	addi	a5, a5, 0x2ee
   143b8:      	add	s4, s9, a4
   143bc:      	add	a5, a5, t1
   143be:      	xor	s0, a6, a4
   143c2:      	xor	a0, a0, a3
   143c4:      	add	a2, a2, a5
   143c6:      	add	a0, a0, a1
   143c8:      	add	s1, a2, t0
   143cc:      	add	t5, a0, a2
   143d0:      	and	s0, s0, s1
   143d2:      	srliw	a0, s1, 0x6
   143d6:      	slli	a2, s1, 0x1a
   143da:      	srliw	a3, s1, 0xb
   143de:      	slli	a5, s1, 0x15
   143e2:      	xor	t1, s0, a4
   143e6:      	srliw	s0, s1, 0x19
   143ea:      	or	a0, a0, a2
   143ec:      	slli	a2, s1, 0x7
   143f0:      	or	a3, a3, a5
   143f2:      	srliw	a5, t5, 0x2
   143f6:      	or	t0, a2, s0
   143fa:      	slli	a1, t5, 0x1e
   143fe:      	or	a1, a1, a5
   14400:      	srliw	a5, t5, 0xd
   14404:      	slli	s0, t5, 0x13
   14408:      	or	a5, a5, s0
   1440a:      	srliw	s0, t5, 0x16
   1440e:      	slli	a2, t5, 0xa
   14412:      	or	a2, a2, s0
   14414:      	xor	s0, t4, t2
   14418:      	and	s0, t5, s0
   1441c:      	and	a4, t4, t2
   14420:      	xor	a4, a4, s0
   14422:      	add	a7, a7, t1
   14424:      	xor	a0, a0, a3
   14426:      	xor	a1, a1, a5
   14428:      	lui	a3, 0x78a56
   1442c:      	addi	a3, a3, 0x36f
   14430:      	ld	s9, 0x88(sp)
   14432:      	add	s9, s9, a6
   14434:      	xor	a0, a0, t0
   14438:      	xor	a5, s1, a6
   1443c:      	add	a3, a3, a7
   1443e:      	xor	a1, a1, a2
   14440:      	add	a3, a3, a0
   14442:      	add	a1, a1, a4
   14444:      	add	a0, a3, t3
   14448:      	add	t1, a1, a3
   1444c:      	and	a5, a5, a0
   1444e:      	srliw	a1, a0, 0x6
   14452:      	slli	a2, a0, 0x1a
   14456:      	srliw	a3, a0, 0xb
   1445a:      	slli	a4, a0, 0x15
   1445e:      	srliw	s0, a0, 0x19
   14462:      	xor	a6, a5, a6
   14466:      	slli	a5, a0, 0x7
   1446a:      	or	t0, a2, a1
   1446e:      	srliw	a2, t1, 0x2
   14472:      	or	a3, a3, a4
   14474:      	slli	a4, t1, 0x1e
   14478:      	or	a7, a5, s0
   1447c:      	srliw	s0, t1, 0xd
   14480:      	or	a2, a2, a4
   14482:      	slli	a4, t1, 0x13
   14486:      	or	a4, a4, s0
   14488:      	srliw	s0, t1, 0x16
   1448c:      	slli	a5, t1, 0xa
   14490:      	or	a5, a5, s0
   14492:      	xor	s0, t5, t4
   14496:      	and	s0, t1, s0
   1449a:      	and	a1, t5, t4
   1449e:      	xor	a1, a1, s0
   144a0:      	add	a6, a6, s4
   144a2:      	xor	a3, t0, a3
   144a6:      	lui	s0, 0x84c88
   144aa:      	addi	s0, s0, -0x7ec
   144ae:      	add	s3, s3, s1
   144b0:      	xor	a2, a2, a4
   144b2:      	xor	a4, a0, s1
   144b6:      	xor	a3, a3, a7
   144ba:      	add	a6, a6, s0
   144bc:      	xor	a2, a2, a5
   144be:      	add	a3, a3, a6
   144c0:      	add	a1, a1, a2
   144c2:      	add	a5, a3, t2
   144c6:      	add	t0, a1, a3
   144ca:      	and	a4, a4, a5
   144cc:      	srliw	a7, a5, 0x6
   144d0:      	slli	a2, a5, 0x1a
   144d4:      	srliw	a3, a5, 0xb
   144d8:      	slli	s0, a5, 0x15
   144dc:      	srliw	a6, a5, 0x19
   144e0:      	slli	a1, a5, 0x7
   144e4:      	xor	a4, a4, s1
   144e6:      	srliw	s1, t0, 0x2
   144ea:      	or	a7, a2, a7
   144ee:      	slli	a2, t0, 0x1e
   144f2:      	or	a3, a3, s0
   144f4:      	srliw	s0, t0, 0xd
   144f8:      	or	a6, a1, a6
   144fc:      	slli	a1, t0, 0x13
   14500:      	or	a2, a2, s1
   14502:      	srliw	s1, t0, 0x16
   14506:      	or	a1, a1, s0
   14508:      	slli	s0, t0, 0xa
   1450c:      	or	t2, s0, s1
   14510:      	xor	s1, t1, t5
   14514:      	and	s1, t0, s1
   14518:      	and	s0, t1, t5
   1451c:      	xor	t3, s1, s0
   14520:      	add	a4, a4, s9
   14522:      	lui	s1, 0x8cc70
   14526:      	addi	s1, s1, 0x208
   1452a:      	ld	s11, 0xe0(sp)
   1452c:      	add	s11, s11, a0
   1452e:      	xor	a3, a7, a3
   14532:      	xor	s0, a5, a0
   14536:      	xor	a1, a1, a2
   14538:      	xor	a7, t0, t1
   1453c:      	xor	a3, a3, a6
   14540:      	add	a4, a4, s1
   14542:      	xor	a1, a1, t2
   14546:      	add	a4, a4, a3
   14548:      	add	a1, a1, t3
   1454a:      	add	a3, a4, t4
   1454e:      	add	a4, a4, a1
   14550:      	and	s0, s0, a3
   14552:      	srliw	a6, a3, 0x6
   14556:      	slli	s1, a3, 0x1a
   1455a:      	srliw	t3, a3, 0xb
   1455e:      	slli	a1, a3, 0x15
   14562:      	srliw	t2, a3, 0x19
   14566:      	slli	a2, a3, 0x7
   1456a:      	xor	t4, s0, a0
   1456e:      	srliw	s0, a4, 0x2
   14572:      	or	a6, s1, a6
   14576:      	slli	s1, a4, 0x1e
   1457a:      	or	a1, a1, t3
   1457e:      	srliw	a0, a4, 0xd
   14582:      	or	t2, a2, t2
   14586:      	slli	a2, a4, 0x13
   1458a:      	or	s0, s0, s1
   1458c:      	srliw	s1, a4, 0x16
   14590:      	or	a2, a2, a0
   14592:      	slli	a0, a4, 0xa
   14596:      	or	t3, a0, s1
   1459a:      	and	a0, t0, t1
   1459e:      	and	s1, a4, a7
   145a2:      	xor	s1, s1, a0
   145a4:      	add	t4, t4, s3
   145a6:      	lui	a0, 0x90bf0
   145aa:      	addi	a0, a0, -0x6
   145ac:      	ld	t6, 0x190(sp)
   145ae:      	add	t6, t6, a5
   145b0:      	xor	a1, a6, a1
   145b4:      	xor	a6, a3, a5
   145b8:      	xor	a2, a2, s0
   145ba:      	xor	a7, a4, t0
   145be:      	xor	a1, a1, t2
   145c2:      	add	a0, a0, t4
   145c4:      	xor	a2, a2, t3
   145c8:      	add	a0, a0, a1
   145ca:      	add	s1, s1, a2
   145cc:      	add	a2, a0, t5
   145d0:      	add	s1, s1, a0
   145d2:      	and	a0, a2, a6
   145d6:      	srliw	t2, a2, 0x6
   145da:      	slli	s0, a2, 0x1a
   145de:      	srliw	a6, a2, 0xb
   145e2:      	slli	a1, a2, 0x15
   145e6:      	srliw	t3, a2, 0x19
   145ea:      	slli	t4, a2, 0x7
   145ee:      	xor	t5, a0, a5
   145f2:      	srliw	a5, s1, 0x2
   145f6:      	or	t2, s0, t2
   145fa:      	slli	s0, s1, 0x1e
   145fe:      	or	a6, a1, a6
   14602:      	srliw	a1, s1, 0xd
   14606:      	or	t3, t4, t3
   1460a:      	slli	a0, s1, 0x13
   1460e:      	or	s0, s0, a5
   14610:      	srliw	a5, s1, 0x16
   14614:      	or	a0, a0, a1
   14616:      	slli	a1, s1, 0xa
   1461a:      	or	t4, a1, a5
   1461e:      	and	a5, a4, t0
   14622:      	and	a1, s1, a7
   14626:      	xor	s3, a1, a5
   1462a:      	add	t5, t5, s11
   1462c:      	lui	a5, 0xa4507
   14630:      	addi	a5, a5, -0x315
   14634:      	ld	a7, 0x1a0(sp)
   14636:      	add	a7, a7, a4
   14638:      	ld	s2, 0x198(sp)
   1463a:      	add	s2, s2, a3
   1463c:      	xor	a1, t2, a6
   14640:      	xor	t2, a2, a3
   14644:      	ld	a6, 0x110(sp)
   14646:      	add	a6, a6, s1
   14648:      	xor	a0, a0, s0
   1464a:      	xor	s5, s1, a4
   1464e:      	and	s4, s1, a4
   14652:      	xor	a1, a1, t3
   14656:      	add	a5, a5, t5
   14658:      	xor	a0, a0, t4
   1465c:      	add	a1, a1, a5
   1465e:      	add	a0, a0, s3
   14660:      	add	t1, t1, a1
   14662:      	add	a0, a0, a1
   14664:      	and	a5, t1, t2
   14668:      	srliw	t3, t1, 0x6
   1466c:      	slli	a4, t1, 0x1a
   14670:      	srliw	t5, t1, 0xb
   14674:      	slli	a1, t1, 0x15
   14678:      	srliw	t4, t1, 0x19
   1467c:      	slli	s0, t1, 0x7
   14680:      	ld	t2, 0x108(sp)
   14682:      	add	t2, t2, a0
   14684:      	xor	s3, a5, a3
   14688:      	srliw	a5, a0, 0x2
   1468c:      	or	t3, a4, t3
   14690:      	slli	a3, a0, 0x1e
   14694:      	or	t5, a1, t5
   14698:      	srliw	a4, a0, 0xd
   1469c:      	or	t4, s0, t4
   146a0:      	slli	a1, a0, 0x13
   146a4:      	or	a3, a3, a5
   146a6:      	srliw	a5, a0, 0x16
   146aa:      	and	s0, a0, s5
   146ae:      	or	a1, a1, a4
   146b0:      	xor	a4, a0, s1
   146b4:      	and	s5, a0, s1
   146b8:      	slli	a0, a0, 0xa
   146ba:      	or	a0, a0, a5
   146bc:      	xor	a5, s0, s4
   146c0:      	add	t6, t6, s3
   146c2:      	xor	s0, t3, t5
   146c6:      	xor	a1, a1, a3
   146c8:      	xor	a3, s0, t4
   146cc:      	lui	s0, 0xbef9a
   146d0:      	addi	s0, s0, 0x3f7
   146d4:      	add	t6, t6, s0
   146d6:      	xor	a0, a0, a1
   146d8:      	ld	t3, 0x118(sp)
   146da:      	add	t3, t3, a2
   146dc:      	ld	t4, 0x1b0(sp)
   146de:      	add	t4, t4, t1
   146e0:      	xor	a1, t1, a2
   146e4:      	add	a3, a3, t6
   146e6:      	add	a0, a0, a5
   146e8:      	add	t0, t0, a3
   146ea:      	add	a3, a3, a0
   146ec:      	ld	t1, 0x1a8(sp)
   146ee:      	add	t1, t1, t0
   146f0:      	and	a1, t0, a1
   146f4:      	srliw	a5, t0, 0x6
   146f8:      	slli	s0, t0, 0x1a
   146fc:      	srliw	a0, t0, 0xb
   14700:      	xor	t6, a1, a2
   14704:      	slli	a2, t0, 0x15
   14708:      	or	a5, a5, s0
   1470a:      	srliw	s0, t0, 0x19
   1470e:      	slli	t0, t0, 0x7
   14710:      	ld	t5, 0x100(sp)
   14712:      	add	t5, t5, a3
   14714:      	and	a4, a4, a3
   14716:      	or	a2, a2, a0
   14718:      	srliw	a0, a3, 0x2
   1471c:      	or	s0, t0, s0
   14720:      	slli	s1, a3, 0x1e
   14724:      	xor	a4, a4, s5
   14728:      	srliw	a1, a3, 0xd
   1472c:      	or	s1, s1, a0
   1472e:      	slli	a0, a3, 0x13
   14732:      	or	a0, a0, a1
   14734:      	srliw	a1, a3, 0x16
   14738:      	slli	a3, a3, 0xa
   1473a:      	or	a1, a1, a3
   1473c:      	add	t6, t6, s2
   1473e:      	xor	a2, a2, a5
   14740:      	ld	a3, 0xf8(sp)
   14742:      	add	a4, a4, a3
   14744:      	xor	a0, a0, s1
   14746:      	xor	a2, a2, s0
   14748:      	lui	a3, 0xc6718
   1474c:      	addi	a3, a3, -0x70e
   14750:      	add	a3, a3, t6
   14752:      	xor	a0, a0, a1
   14754:      	add	a2, a2, a3
   14756:      	add	a0, a0, a4
   14758:      	add	a7, a7, a2
   1475a:      	add	a0, a0, a2
   1475c:      	ld	a1, 0x90(sp)
   1475e:      	sw	a0, 0x0(a1)
   14760:      	sw	t5, 0x4(a1)
   14764:      	sw	t2, 0x8(a1)
   14768:      	sw	a6, 0xc(a1)
   1476c:      	sw	a7, 0x10(a1)
   14770:      	sw	t1, 0x14(a1)
   14774:      	sw	t4, 0x18(a1)
   14778:      	sw	t3, 0x1c(a1)
   1477c:      	ld	ra, 0x238(sp)
   14780:      	ld	s0, 0x230(sp)
   14784:      	ld	s1, 0x228(sp)
   14788:      	ld	s2, 0x220(sp)
   1478c:      	ld	s3, 0x218(sp)
   14790:      	ld	s4, 0x210(sp)
   14794:      	ld	s5, 0x208(sp)
   14798:      	ld	s6, 0x200(sp)
   1479c:      	ld	s7, 0x1f8(sp)
   1479e:      	ld	s8, 0x1f0(sp)
   147a0:      	ld	s9, 0x1e8(sp)
   147a2:      	ld	s10, 0x1e0(sp)
   147a4:      	ld	s11, 0x1d8(sp)
   147a6:      	addi	sp, sp, 0x240
   147aa:      	ret

00000000000147ac <memset>:
   147ac:      	addi	sp, sp, -0x10
   147ae:      	sd	ra, 0x8(sp)
   147b0:      	sd	s0, 0x0(sp)
   147b2:      	addi	s0, sp, 0x10
   147b4:      	li	a3, 0x10
   147b6:      	bltu	a2, a3, 0x14824 <memset+0x78>
   147ba:      	negw	a3, a0
   147be:      	andi	a6, a3, 0x7
   147c2:      	add	a4, a0, a6
   147c6:      	bgeu	a0, a4, 0x147d8 <memset+0x2c>
   147ca:      	mv	a5, a6
   147cc:      	mv	a3, a0
   147ce:      	sb	a1, 0x0(a3)
   147d2:      	addi	a5, a5, -0x1
   147d4:      	addi	a3, a3, 0x1
   147d6:      	bnez	a5, 0x147ce <memset+0x22>
   147d8:      	sub	a2, a2, a6
   147dc:      	andi	a3, a2, -0x8
   147e0:      	add	a3, a3, a4
   147e2:      	bgeu	a4, a3, 0x14808 <memset+0x5c>
   147e6:      	slli	a6, a1, 0x38
   147ea:      	lui	a5, 0x10101
   147ee:      	slli	a5, a5, 0x4
   147f0:      	addi	a5, a5, 0x100
   147f4:      	mulhu	a6, a6, a5
   147f8:      	slli	a5, a6, 0x20
   147fc:      	or	a5, a5, a6
   14800:      	sd	a5, 0x0(a4)
   14802:      	addi	a4, a4, 0x8
   14804:      	bltu	a4, a3, 0x14800 <memset+0x54>
   14808:      	andi	a2, a2, 0x7
   1480a:      	add	a4, a3, a2
   1480e:      	bgeu	a3, a4, 0x1481c <memset+0x70>
   14812:      	sb	a1, 0x0(a3)
   14816:      	addi	a2, a2, -0x1
   14818:      	addi	a3, a3, 0x1
   1481a:      	bnez	a2, 0x14812 <memset+0x66>
   1481c:      	ld	ra, 0x8(sp)
   1481e:      	ld	s0, 0x0(sp)
   14820:      	addi	sp, sp, 0x10
   14822:      	ret
   14824:      	mv	a3, a0
   14826:      	add	a4, a0, a2
   1482a:      	bltu	a0, a4, 0x14812 <memset+0x66>
   1482e:      	j	0x1481c <memset+0x70>

0000000000014830 <memcpy>:
   14830:      	addi	sp, sp, -0x10
   14832:      	sd	ra, 0x8(sp)
   14834:      	sd	s0, 0x0(sp)
   14836:      	addi	s0, sp, 0x10
   14838:      	ld	ra, 0x8(sp)
   1483a:      	ld	s0, 0x0(sp)
   1483c:      	addi	sp, sp, 0x10
   1483e:      	auipc	t1, 0x0
   14842:      	jr	0x8(t1) <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6>

0000000000014846 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6>:
   14846:      	addi	sp, sp, -0x20
   14848:      	sd	ra, 0x18(sp)
   1484a:      	sd	s0, 0x10(sp)
   1484c:      	sd	s1, 0x8(sp)
   1484e:      	addi	s0, sp, 0x20
   14850:      	li	a3, 0x10
   14852:      	bltu	a2, a3, 0x148b6 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x70>
   14856:      	negw	a3, a0
   1485a:      	andi	a6, a3, 0x7
   1485e:      	add	t6, a0, a6
   14862:      	bgeu	a0, t6, 0x1487c <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x36>
   14866:      	mv	a4, a6
   14868:      	mv	a3, a0
   1486a:      	mv	a5, a1
   1486c:      	lbu	a7, 0x0(a5)
   14870:      	addi	a4, a4, -0x1
   14872:      	sb	a7, 0x0(a3)
   14876:      	addi	a3, a3, 0x1
   14878:      	addi	a5, a5, 0x1
   1487a:      	bnez	a4, 0x1486c <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x26>
   1487c:      	add	a1, a1, a6
   1487e:      	sub	s1, a2, a6
   14882:      	andi	a4, s1, -0x8
   14886:      	andi	a6, a1, 0x7
   1488a:      	add	a3, t6, a4
   1488e:      	bnez	a6, 0x148da <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x94>
   14892:      	bgeu	t6, a3, 0x148a6 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x60>
   14896:      	mv	a5, a1
   14898:      	ld	a2, 0x0(a5)
   1489a:      	sd	a2, 0x0(t6)
   1489e:      	addi	t6, t6, 0x8
   148a0:      	addi	a5, a5, 0x8
   148a2:      	bltu	t6, a3, 0x14898 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x52>
   148a6:      	add	a1, a1, a4
   148a8:      	andi	a2, s1, 0x7
   148ac:      	add	a4, a3, a2
   148b0:      	bltu	a3, a4, 0x148c0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x7a>
   148b4:      	j	0x148d0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x8a>
   148b6:      	mv	a3, a0
   148b8:      	add	a4, a0, a2
   148bc:      	bgeu	a0, a4, 0x148d0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x8a>
   148c0:      	lbu	a4, 0x0(a1)
   148c4:      	addi	a2, a2, -0x1
   148c6:      	sb	a4, 0x0(a3)
   148ca:      	addi	a3, a3, 0x1
   148cc:      	addi	a1, a1, 0x1
   148ce:      	bnez	a2, 0x148c0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x7a>
   148d0:      	ld	ra, 0x18(sp)
   148d2:      	ld	s0, 0x10(sp)
   148d4:      	ld	s1, 0x8(sp)
   148d6:      	addi	sp, sp, 0x20
   148d8:      	ret
   148da:      	li	a7, 0x0
   148dc:      	li	a2, 0x8
   148de:      	sd	zero, -0x20(s0)
   148e2:      	sub	t1, a2, a6
   148e6:      	addi	a2, s0, -0x20
   148ea:      	andi	a5, t1, 0x1
   148ee:      	or	t0, a2, a6
   148f2:      	bnez	a5, 0x14946 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x100>
   148f4:      	andi	a2, t1, 0x2
   148f8:      	bnez	a2, 0x14956 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x110>
   148fa:      	andi	a2, t1, 0x4
   148fe:      	bnez	a2, 0x1496e <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x128>
   14900:      	ld	t4, -0x20(s0)
   14904:      	slli	a7, a6, 0x3
   14908:      	addi	a2, t6, 0x8
   1490c:      	sub	t5, a1, a6
   14910:      	bgeu	a2, a3, 0x1498e <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x148>
   14914:      	negw	a2, a7
   14918:      	andi	t2, a2, 0x38
   1491c:      	ld	t0, 0x8(t5)
   14920:      	addi	t3, t5, 0x8
   14924:      	srl	a2, t4, a7
   14928:      	addi	t1, t6, 0x8
   1492c:      	sll	a5, t0, t2
   14930:      	or	a2, a2, a5
   14932:      	addi	a5, t6, 0x10
   14936:      	sd	a2, 0x0(t6)
   1493a:      	mv	t6, t1
   1493c:      	mv	t5, t3
   1493e:      	mv	t4, t0
   14940:      	bltu	a5, a3, 0x1491c <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xd6>
   14944:      	j	0x14994 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x14e>
   14946:      	lbu	a2, 0x0(a1)
   1494a:      	sb	a2, 0x0(t0)
   1494e:      	li	a7, 0x1
   14950:      	andi	a2, t1, 0x2
   14954:      	beqz	a2, 0x148fa <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xb4>
   14956:      	add	a2, a1, a7
   1495a:      	lh	a2, 0x0(a2)
   1495e:      	add	a5, t0, a7
   14962:      	sh	a2, 0x0(a5)
   14966:      	addi	a7, a7, 0x2
   14968:      	andi	a2, t1, 0x4
   1496c:      	beqz	a2, 0x14900 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xba>
   1496e:      	add	a2, a1, a7
   14972:      	lw	a2, 0x0(a2)
   14974:      	add	a7, a7, t0
   14976:      	sw	a2, 0x0(a7)
   1497a:      	ld	t4, -0x20(s0)
   1497e:      	slli	a7, a6, 0x3
   14982:      	addi	a2, t6, 0x8
   14986:      	sub	t5, a1, a6
   1498a:      	bltu	a2, a3, 0x14914 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0xce>
   1498e:      	mv	t0, t4
   14990:      	mv	t3, t5
   14992:      	mv	t1, t6
   14994:      	li	a5, 0x0
   14996:      	addi	t2, t3, 0x8
   1499a:      	li	a2, 0x4
   1499c:      	sd	zero, -0x20(s0)
   149a0:      	bgeu	a6, a2, 0x149ea <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x1a4>
   149a4:      	andi	a2, a1, 0x2
   149a8:      	bnez	a2, 0x149fa <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x1b4>
   149aa:      	andi	a2, a1, 0x1
   149ae:      	beqz	a2, 0x149c0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x17a>
   149b0:      	add	t2, t2, a5
   149b2:      	lbu	a6, 0x0(t2)
   149b6:      	addi	a2, s0, -0x20
   149ba:      	or	a2, a2, a5
   149bc:      	sb	a6, 0x0(a2)
   149c0:      	ld	a6, -0x20(s0)
   149c4:      	srl	a5, t0, a7
   149c8:      	negw	a2, a7
   149cc:      	andi	a2, a2, 0x38
   149d0:      	sll	a2, a6, a2
   149d4:      	or	a2, a2, a5
   149d6:      	sd	a2, 0x0(t1)
   149da:      	add	a1, a1, a4
   149dc:      	andi	a2, s1, 0x7
   149e0:      	add	a4, a3, a2
   149e4:      	bltu	a3, a4, 0x148c0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x7a>
   149e8:      	j	0x148d0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x8a>
   149ea:      	lw	a2, 0x0(t2)
   149ee:      	sw	a2, -0x20(s0)
   149f2:      	li	a5, 0x4
   149f4:      	andi	a2, a1, 0x2
   149f8:      	beqz	a2, 0x149aa <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x164>
   149fa:      	add	a2, t2, a5
   149fe:      	lh	a6, 0x0(a2)
   14a02:      	addi	a2, s0, -0x20
   14a06:      	or	a2, a2, a5
   14a08:      	sh	a6, 0x0(a2)
   14a0c:      	addi	a5, a5, 0x2
   14a0e:      	andi	a2, a1, 0x1
   14a12:      	bnez	a2, 0x149b0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x16a>
   14a14:      	j	0x149c0 <compiler_builtins::mem::memcpy::hf6c03fdc5531bce6+0x17a>
