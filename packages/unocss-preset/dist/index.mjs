//#endregion
//#region src/index.ts
const wabouUtilityManifest = {
	version: 5,
	spacing: {
		"0": 0,
		"0.5": 2,
		"1": 4,
		"1.5": 6,
		"10": 40,
		"11": 44,
		"12": 48,
		"14": 56,
		"16": 64,
		"18": 72,
		"2": 8,
		"2.5": 10,
		"20": 80,
		"22": 88,
		"24": 96,
		"28": 112,
		"3": 12,
		"3.5": 14,
		"32": 128,
		"36": 144,
		"4": 16,
		"40": 160,
		"44": 176,
		"48": 192,
		"5": 20,
		"52": 208,
		"56": 224,
		"6": 24,
		"60": 240,
		"64": 256,
		"7": 28,
		"72": 288,
		"8": 32,
		"80": 320,
		"9": 36,
		"96": 384
	},
	colors: {
		"amber-100": 4277389311,
		"amber-200": 4259744511,
		"amber-300": 4241706495,
		"amber-400": 4223608063,
		"amber-50": 4294700031,
		"amber-500": 4120775679,
		"amber-600": 3648456447,
		"amber-700": 3025340927,
		"amber-800": 2453671679,
		"amber-900": 2016743423,
		"amber-950": 1159332863,
		"black": 255,
		"blue-100": 3689611007,
		"blue-200": 3218865919,
		"blue-300": 2479226367,
		"blue-400": 1621490431,
		"blue-50": 4025942015,
		"blue-500": 998438655,
		"blue-600": 627305471,
		"blue-700": 491706623,
		"blue-800": 507555839,
		"blue-900": 507153151,
		"blue-950": 388322559,
		"cyan-100": 3489332991,
		"cyan-200": 2784230655,
		"cyan-300": 1743321599,
		"cyan-400": 584314623,
		"cyan-50": 3976134655,
		"cyan-500": 112645375,
		"cyan-600": 143766271,
		"cyan-700": 242520319,
		"cyan-800": 358512127,
		"cyan-900": 374236159,
		"cyan-950": 137577727,
		"emerald-100": 3522881023,
		"emerald-200": 2817773823,
		"emerald-300": 1860679679,
		"emerald-400": 886282751,
		"emerald-50": 3976066559,
		"emerald-500": 280592895,
		"emerald-600": 93743615,
		"emerald-700": 74995711,
		"emerald-800": 106907391,
		"emerald-900": 105790463,
		"emerald-950": 36446975,
		"fuchsia-100": 4209573887,
		"fuchsia-200": 4124114687,
		"fuchsia-300": 4037803263,
		"fuchsia-400": 3900307967,
		"fuchsia-50": 4260691967,
		"fuchsia-500": 3645304831,
		"fuchsia-600": 3223770111,
		"fuchsia-700": 2719789055,
		"fuchsia-800": 2249822207,
		"fuchsia-900": 1880782335,
		"fuchsia-950": 1241796351,
		"gray-100": 4092917503,
		"gray-200": 3857181695,
		"gray-300": 3520453631,
		"gray-400": 2627973119,
		"gray-50": 4193975295,
		"gray-500": 1802666239,
		"gray-600": 1263887359,
		"gray-700": 927027711,
		"gray-800": 522795007,
		"gray-900": 286795775,
		"gray-950": 50795263,
		"green-100": 3707561983,
		"green-200": 3153580287,
		"green-300": 2263854335,
		"green-400": 1256095999,
		"green-50": 4043175167,
		"green-500": 583360255,
		"green-600": 379800319,
		"green-700": 360726015,
		"green-800": 375731455,
		"green-900": 340995583,
		"green-950": 86906623,
		"indigo-100": 3773300735,
		"indigo-200": 3352493823,
		"indigo-300": 2780101887,
		"indigo-400": 2173499647,
		"indigo-50": 4008902655,
		"indigo-500": 1667691007,
		"indigo-600": 1330046463,
		"indigo-700": 1127795455,
		"indigo-800": 925934591,
		"indigo-900": 825131519,
		"indigo-950": 505105407,
		"lime-100": 3975990271,
		"lime-200": 3657014783,
		"lime-300": 3203556607,
		"lime-400": 2749773311,
		"lime-50": 4160677887,
		"lime-500": 2227967743,
		"lime-600": 1705184767,
		"lime-700": 1299976191,
		"lime-800": 1063391999,
		"lime-900": 911414527,
		"lime-950": 439223807,
		"neutral-100": 4126537215,
		"neutral-200": 3857049087,
		"neutral-300": 3570717951,
		"neutral-400": 2745410559,
		"neutral-50": 4210752255,
		"neutral-500": 1936946175,
		"neutral-600": 1381126911,
		"neutral-700": 1077952767,
		"neutral-800": 640034559,
		"neutral-900": 387389439,
		"neutral-950": 168430335,
		"orange-100": 4293776895,
		"orange-200": 4275546879,
		"orange-300": 4256855295,
		"orange-400": 4220665087,
		"orange-50": 4294438399,
		"orange-500": 4185069311,
		"orange-600": 3931639039,
		"orange-700": 3259043071,
		"orange-800": 2587103999,
		"orange-900": 2083328767,
		"orange-950": 1125386239,
		"pink-100": 4243059711,
		"pink-200": 4224706815,
		"pink-300": 4188591359,
		"pink-400": 4101158655,
		"pink-50": 4260559103,
		"pink-500": 3964180991,
		"pink-600": 3676796927,
		"pink-700": 3189267967,
		"pink-800": 2635550207,
		"pink-900": 2199405567,
		"pink-950": 1342645503,
		"purple-100": 4092133375,
		"purple-200": 3923116031,
		"purple-300": 3635740415,
		"purple-400": 3229940991,
		"purple-50": 4210425855,
		"purple-500": 2824206335,
		"purple-600": 2469653247,
		"purple-700": 2116210431,
		"purple-800": 1797368063,
		"purple-900": 1478264831,
		"purple-950": 990340351,
		"red-100": 4276282111,
		"red-200": 4274703103,
		"red-300": 4238714367,
		"red-400": 4168184319,
		"red-50": 4277334783,
		"red-500": 4014228735,
		"red-600": 3693487871,
		"red-700": 3105627391,
		"red-800": 2568690687,
		"red-900": 2132614655,
		"red-950": 1158286079,
		"rose-100": 4293191423,
		"rose-200": 4274902015,
		"rose-300": 4255428607,
		"rose-400": 4218521087,
		"rose-50": 4294046463,
		"rose-500": 4097793791,
		"rose-600": 3776792831,
		"rose-700": 3188866303,
		"rose-800": 2668771839,
		"rose-900": 2282960895,
		"rose-950": 1275402751,
		"sky-100": 3774021375,
		"sky-200": 3135700479,
		"sky-300": 2111044863,
		"sky-400": 951974143,
		"sky-50": 4042915839,
		"sky-500": 245754367,
		"sky-600": 42256383,
		"sky-700": 57254399,
		"sky-800": 123307519,
		"sky-900": 206204671,
		"sky-950": 137316863,
		"slate-100": 4059429375,
		"slate-200": 3806916863,
		"slate-300": 3419791871,
		"slate-400": 2493757695,
		"slate-50": 4177198335,
		"slate-500": 1685359615,
		"slate-600": 1196780031,
		"slate-700": 859919871,
		"slate-800": 506018815,
		"slate-900": 253176575,
		"slate-950": 33953791,
		"stone-100": 4126536959,
		"stone-200": 3890603263,
		"stone-300": 3604206079,
		"stone-400": 2829229823,
		"stone-50": 4210751999,
		"stone-500": 2020699391,
		"stone-600": 1465077503,
		"stone-700": 1145060607,
		"stone-800": 690300159,
		"stone-900": 471406591,
		"stone-950": 201984511,
		"teal-100": 3439063551,
		"teal-200": 2583094527,
		"teal-300": 1592448255,
		"teal-400": 768917503,
		"teal-50": 4043176703,
		"teal-500": 347645695,
		"teal-600": 227838207,
		"teal-700": 259419903,
		"teal-800": 291396095,
		"teal-900": 323898111,
		"teal-950": 70201087,
		"transparent": 0,
		"violet-100": 3991535359,
		"violet-200": 3721854719,
		"violet-300": 3300261375,
		"violet-400": 2810968831,
		"violet-50": 4126408703,
		"violet-500": 2338125567,
		"violet-600": 2084236799,
		"violet-700": 1831393791,
		"violet-800": 1528936191,
		"violet-900": 1277007359,
		"violet-950": 772826623,
		"white": 4294967295,
		"yellow-100": 4277781503,
		"yellow-200": 4277177087,
		"yellow-300": 4259334143,
		"yellow-400": 4207678975,
		"yellow-50": 4277987583,
		"yellow-500": 3937601791,
		"yellow-600": 3398042879,
		"yellow-700": 2707556351,
		"yellow-800": 2236419839,
		"yellow-900": 1899959039,
		"yellow-950": 1109395199,
		"zinc-100": 4109694463,
		"zinc-200": 3840206847,
		"zinc-300": 3570718975,
		"zinc-400": 2711726847,
		"zinc-50": 4210752255,
		"zinc-500": 1903262463,
		"zinc-600": 1381129215,
		"zinc-700": 1061111551,
		"zinc-800": 656878335,
		"zinc-900": 404233215,
		"zinc-950": 151587839
	},
	staticUtilities: {
		"absolute": [{
			"property": "position",
			"value": {
				"type": "keyword",
				"value": "absolute"
			}
		}],
		"aspect-square": [{
			"property": "aspect-ratio",
			"value": {
				"type": "number",
				"value": 1
			}
		}],
		"aspect-video": [{
			"property": "aspect-ratio",
			"value": {
				"type": "number",
				"value": 1.7777778
			}
		}],
		"block": [{
			"property": "display",
			"value": {
				"type": "keyword",
				"value": "block"
			}
		}],
		"border": [{
			"property": "border-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}],
		"border-0": [{
			"property": "border-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 0
				}
			}
		}],
		"border-2": [{
			"property": "border-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 2
				}
			}
		}],
		"border-4": [{
			"property": "border-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 4
				}
			}
		}],
		"border-8": [{
			"property": "border-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 8
				}
			}
		}],
		"border-b": [{
			"property": "border-bottom-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}],
		"border-l": [{
			"property": "border-left-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}],
		"border-r": [{
			"property": "border-right-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}],
		"border-t": [{
			"property": "border-top-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}],
		"box-border": [{
			"property": "box-sizing",
			"value": {
				"type": "keyword",
				"value": "border-box"
			}
		}],
		"content-around": [{
			"property": "align-content",
			"value": {
				"type": "keyword",
				"value": "space-around"
			}
		}],
		"content-between": [{
			"property": "align-content",
			"value": {
				"type": "keyword",
				"value": "space-between"
			}
		}],
		"content-center": [{
			"property": "align-content",
			"value": {
				"type": "keyword",
				"value": "center"
			}
		}],
		"content-end": [{
			"property": "align-content",
			"value": {
				"type": "keyword",
				"value": "flex-end"
			}
		}],
		"content-start": [{
			"property": "align-content",
			"value": {
				"type": "keyword",
				"value": "flex-start"
			}
		}],
		"cursor-auto": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "auto"
			}
		}],
		"cursor-crosshair": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "crosshair"
			}
		}],
		"cursor-default": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "default"
			}
		}],
		"cursor-ew-resize": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "ew-resize"
			}
		}],
		"cursor-move": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "move"
			}
		}],
		"cursor-not-allowed": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "not-allowed"
			}
		}],
		"cursor-ns-resize": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "ns-resize"
			}
		}],
		"cursor-pointer": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "pointer"
			}
		}],
		"cursor-text": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "text"
			}
		}],
		"cursor-wait": [{
			"property": "cursor",
			"value": {
				"type": "keyword",
				"value": "wait"
			}
		}],
		"flex": [{
			"property": "display",
			"value": {
				"type": "keyword",
				"value": "flex"
			}
		}],
		"flex-1": [
			{
				"property": "flex-grow",
				"value": {
					"type": "number",
					"value": 1
				}
			},
			{
				"property": "flex-shrink",
				"value": {
					"type": "number",
					"value": 1
				}
			},
			{
				"property": "flex-basis",
				"value": {
					"type": "length",
					"value": {
						"unit": "percent",
						"value": 0
					}
				}
			}
		],
		"flex-col": [{
			"property": "flex-direction",
			"value": {
				"type": "keyword",
				"value": "column"
			}
		}],
		"flex-none": [
			{
				"property": "flex-grow",
				"value": {
					"type": "number",
					"value": 0
				}
			},
			{
				"property": "flex-shrink",
				"value": {
					"type": "number",
					"value": 0
				}
			},
			{
				"property": "flex-basis",
				"value": {
					"type": "keyword",
					"value": "auto"
				}
			}
		],
		"flex-nowrap": [{
			"property": "flex-wrap",
			"value": {
				"type": "keyword",
				"value": "nowrap"
			}
		}],
		"flex-row": [{
			"property": "flex-direction",
			"value": {
				"type": "keyword",
				"value": "row"
			}
		}],
		"flex-wrap": [{
			"property": "flex-wrap",
			"value": {
				"type": "keyword",
				"value": "wrap"
			}
		}],
		"font-bold": [{
			"property": "font-weight",
			"value": {
				"type": "number",
				"value": 700
			}
		}],
		"font-extrabold": [{
			"property": "font-weight",
			"value": {
				"type": "number",
				"value": 800
			}
		}],
		"font-medium": [{
			"property": "font-weight",
			"value": {
				"type": "number",
				"value": 500
			}
		}],
		"font-mono": [{
			"property": "font-family",
			"value": {
				"type": "keyword",
				"value": "monospace"
			}
		}],
		"font-normal": [{
			"property": "font-weight",
			"value": {
				"type": "number",
				"value": 400
			}
		}],
		"font-sans": [{
			"property": "font-family",
			"value": {
				"type": "keyword",
				"value": "sans-serif"
			}
		}],
		"font-semibold": [{
			"property": "font-weight",
			"value": {
				"type": "number",
				"value": 600
			}
		}],
		"grid": [{
			"property": "display",
			"value": {
				"type": "keyword",
				"value": "grid"
			}
		}],
		"grid-cols-1": [{
			"property": "grid-template-columns",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 1
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-cols-12": [{
			"property": "grid-template-columns",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 12
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-cols-2": [{
			"property": "grid-template-columns",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 2
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-cols-3": [{
			"property": "grid-template-columns",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 3
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-cols-4": [{
			"property": "grid-template-columns",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 4
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-cols-6": [{
			"property": "grid-template-columns",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 6
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-rows-1": [{
			"property": "grid-template-rows",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 1
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-rows-2": [{
			"property": "grid-template-rows",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 2
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-rows-3": [{
			"property": "grid-template-rows",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 3
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grid-rows-4": [{
			"property": "grid-template-rows",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"count": {
							"type": "number",
							"value": 4
						},
						"kind": {
							"type": "keyword",
							"value": "repeat"
						},
						"values": {
							"type": "list",
							"values": [{
								"type": "record",
								"fields": {
									"kind": {
										"type": "keyword",
										"value": "breadth"
									},
									"value": {
										"type": "record",
										"fields": {
											"kind": {
												"type": "keyword",
												"value": "flex"
											},
											"value": {
												"type": "number",
												"value": 1
											}
										}
									}
								}
							}]
						}
					}
				}]
			}
		}],
		"grow": [{
			"property": "flex-grow",
			"value": {
				"type": "number",
				"value": 1
			}
		}],
		"grow-0": [{
			"property": "flex-grow",
			"value": {
				"type": "number",
				"value": 0
			}
		}],
		"hidden": [{
			"property": "display",
			"value": {
				"type": "keyword",
				"value": "none"
			}
		}],
		"inline-flex": [{
			"property": "display",
			"value": {
				"type": "keyword",
				"value": "flex"
			}
		}],
		"items-baseline": [{
			"property": "align-items",
			"value": {
				"type": "keyword",
				"value": "baseline"
			}
		}],
		"items-center": [{
			"property": "align-items",
			"value": {
				"type": "keyword",
				"value": "center"
			}
		}],
		"items-end": [{
			"property": "align-items",
			"value": {
				"type": "keyword",
				"value": "flex-end"
			}
		}],
		"items-start": [{
			"property": "align-items",
			"value": {
				"type": "keyword",
				"value": "flex-start"
			}
		}],
		"items-stretch": [{
			"property": "align-items",
			"value": {
				"type": "keyword",
				"value": "stretch"
			}
		}],
		"justify-around": [{
			"property": "justify-content",
			"value": {
				"type": "keyword",
				"value": "space-around"
			}
		}],
		"justify-between": [{
			"property": "justify-content",
			"value": {
				"type": "keyword",
				"value": "space-between"
			}
		}],
		"justify-center": [{
			"property": "justify-content",
			"value": {
				"type": "keyword",
				"value": "center"
			}
		}],
		"justify-end": [{
			"property": "justify-content",
			"value": {
				"type": "keyword",
				"value": "flex-end"
			}
		}],
		"justify-evenly": [{
			"property": "justify-content",
			"value": {
				"type": "keyword",
				"value": "space-evenly"
			}
		}],
		"justify-start": [{
			"property": "justify-content",
			"value": {
				"type": "keyword",
				"value": "flex-start"
			}
		}],
		"leading-normal": [{
			"property": "line-height",
			"value": {
				"type": "number",
				"value": 1.5
			}
		}],
		"leading-relaxed": [{
			"property": "line-height",
			"value": {
				"type": "number",
				"value": 1.625
			}
		}],
		"leading-tight": [{
			"property": "line-height",
			"value": {
				"type": "number",
				"value": 1.25
			}
		}],
		"max-w-2xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 672
				}
			}
		}],
		"max-w-3xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 768
				}
			}
		}],
		"max-w-4xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 896
				}
			}
		}],
		"max-w-5xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1024
				}
			}
		}],
		"max-w-6xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1152
				}
			}
		}],
		"max-w-7xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1280
				}
			}
		}],
		"max-w-lg": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 512
				}
			}
		}],
		"max-w-md": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 448
				}
			}
		}],
		"max-w-sm": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 384
				}
			}
		}],
		"max-w-xl": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 576
				}
			}
		}],
		"max-w-xs": [{
			"property": "max-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 320
				}
			}
		}],
		"outline": [{
			"property": "outline-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}, {
			"property": "outline-style",
			"value": {
				"type": "keyword",
				"value": "solid"
			}
		}],
		"outline-0": [{
			"property": "outline-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 0
				}
			}
		}],
		"outline-2": [{
			"property": "outline-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 2
				}
			}
		}, {
			"property": "outline-style",
			"value": {
				"type": "keyword",
				"value": "solid"
			}
		}],
		"outline-4": [{
			"property": "outline-width",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 4
				}
			}
		}, {
			"property": "outline-style",
			"value": {
				"type": "keyword",
				"value": "solid"
			}
		}],
		"outline-offset-0": [{
			"property": "outline-offset",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 0
				}
			}
		}],
		"outline-offset-1": [{
			"property": "outline-offset",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 1
				}
			}
		}],
		"outline-offset-2": [{
			"property": "outline-offset",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 2
				}
			}
		}],
		"outline-offset-4": [{
			"property": "outline-offset",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 4
				}
			}
		}],
		"overflow-auto": [{
			"property": "overflow",
			"value": {
				"type": "keyword",
				"value": "auto"
			}
		}],
		"overflow-hidden": [{
			"property": "overflow",
			"value": {
				"type": "keyword",
				"value": "hidden"
			}
		}],
		"overflow-scroll": [{
			"property": "overflow",
			"value": {
				"type": "keyword",
				"value": "scroll"
			}
		}],
		"overflow-visible": [{
			"property": "overflow",
			"value": {
				"type": "keyword",
				"value": "visible"
			}
		}],
		"overflow-x-auto": [{
			"property": "overflow-x",
			"value": {
				"type": "keyword",
				"value": "auto"
			}
		}],
		"overflow-x-hidden": [{
			"property": "overflow-x",
			"value": {
				"type": "keyword",
				"value": "hidden"
			}
		}],
		"overflow-x-scroll": [{
			"property": "overflow-x",
			"value": {
				"type": "keyword",
				"value": "scroll"
			}
		}],
		"overflow-y-auto": [{
			"property": "overflow-y",
			"value": {
				"type": "keyword",
				"value": "auto"
			}
		}],
		"overflow-y-hidden": [{
			"property": "overflow-y",
			"value": {
				"type": "keyword",
				"value": "hidden"
			}
		}],
		"overflow-y-scroll": [{
			"property": "overflow-y",
			"value": {
				"type": "keyword",
				"value": "scroll"
			}
		}],
		"pointer-events-auto": [{
			"property": "pointer-events",
			"value": {
				"type": "keyword",
				"value": "auto"
			}
		}],
		"pointer-events-none": [{
			"property": "pointer-events",
			"value": {
				"type": "keyword",
				"value": "none"
			}
		}],
		"relative": [{
			"property": "position",
			"value": {
				"type": "keyword",
				"value": "relative"
			}
		}],
		"rotate-45": [{
			"property": "transform-rotate",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"kind": {
							"type": "keyword",
							"value": "rotate"
						},
						"value": {
							"type": "number",
							"value": .7853982
						}
					}
				}]
			}
		}],
		"rounded": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 4
				}
			}
		}],
		"rounded-2xl": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 16
				}
			}
		}],
		"rounded-3xl": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 24
				}
			}
		}],
		"rounded-full": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 9999
				}
			}
		}],
		"rounded-lg": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 8
				}
			}
		}],
		"rounded-md": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 6
				}
			}
		}],
		"rounded-none": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 0
				}
			}
		}],
		"rounded-sm": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 2
				}
			}
		}],
		"rounded-xl": [{
			"property": "border-radius",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 12
				}
			}
		}],
		"scale-150": [{
			"property": "transform-scale",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"kind": {
							"type": "keyword",
							"value": "scale"
						},
						"value": {
							"type": "list",
							"values": [{
								"type": "number",
								"value": 1.5
							}, {
								"type": "number",
								"value": 1.5
							}]
						}
					}
				}]
			}
		}],
		"scale-50": [{
			"property": "transform-scale",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"kind": {
							"type": "keyword",
							"value": "scale"
						},
						"value": {
							"type": "list",
							"values": [{
								"type": "number",
								"value": .5
							}, {
								"type": "number",
								"value": .5
							}]
						}
					}
				}]
			}
		}],
		"select-all": [{
			"property": "user-select",
			"value": {
				"type": "keyword",
				"value": "all"
			}
		}],
		"select-none": [{
			"property": "user-select",
			"value": {
				"type": "keyword",
				"value": "none"
			}
		}],
		"select-text": [{
			"property": "user-select",
			"value": {
				"type": "keyword",
				"value": "text"
			}
		}],
		"self-auto": [{
			"property": "align-self",
			"value": {
				"type": "keyword",
				"value": "auto"
			}
		}],
		"self-center": [{
			"property": "align-self",
			"value": {
				"type": "keyword",
				"value": "center"
			}
		}],
		"self-end": [{
			"property": "align-self",
			"value": {
				"type": "keyword",
				"value": "flex-end"
			}
		}],
		"self-start": [{
			"property": "align-self",
			"value": {
				"type": "keyword",
				"value": "flex-start"
			}
		}],
		"self-stretch": [{
			"property": "align-self",
			"value": {
				"type": "keyword",
				"value": "stretch"
			}
		}],
		"shadow": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176340
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 2
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 1
							}
						}
					}
				}, {
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176351
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": -1
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 1
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 2
							}
						}
					}
				}]
			}
		}],
		"shadow-lg": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176351
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 10
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 6
							}
						}
					}
				}, {
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176361
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": -4
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 4
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 12
							}
						}
					}
				}]
			}
		}],
		"shadow-md": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176346
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 6
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 3
							}
						}
					}
				}, {
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176356
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": -2
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 2.5
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 5
							}
						}
					}
				}]
			}
		}],
		"shadow-none": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": []
			}
		}],
		"shadow-sm": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176346
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 1.5
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 1
							}
						}
					}
				}]
			}
		}],
		"shadow-xl": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176356
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 16
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 10
							}
						}
					}
				}, {
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176366
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": -6
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 6
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 20
							}
						}
					}
				}]
			}
		}],
		"shadow-xs": [{
			"property": "box-shadow",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"color": {
							"type": "color",
							"value": {
								"kind": "literal",
								"rgba": 253176340
							}
						},
						"spread": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"stdDev": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": .75
							}
						},
						"x": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 0
							}
						},
						"y": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 1
							}
						}
					}
				}]
			}
		}],
		"shrink": [{
			"property": "flex-shrink",
			"value": {
				"type": "number",
				"value": 1
			}
		}],
		"shrink-0": [{
			"property": "flex-shrink",
			"value": {
				"type": "number",
				"value": 0
			}
		}],
		"text-2xl": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 24
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 32
				}
			}
		}],
		"text-3xl": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 30
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 36
				}
			}
		}],
		"text-base": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 16
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 24
				}
			}
		}],
		"text-center": [{
			"property": "text-align",
			"value": {
				"type": "keyword",
				"value": "center"
			}
		}],
		"text-ellipsis": [{
			"property": "text-overflow",
			"value": {
				"type": "keyword",
				"value": "ellipsis"
			}
		}],
		"text-left": [{
			"property": "text-align",
			"value": {
				"type": "keyword",
				"value": "left"
			}
		}],
		"text-lg": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 18
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 28
				}
			}
		}],
		"text-right": [{
			"property": "text-align",
			"value": {
				"type": "keyword",
				"value": "right"
			}
		}],
		"text-sm": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 14
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 20
				}
			}
		}],
		"text-xl": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 20
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 28
				}
			}
		}],
		"text-xs": [{
			"property": "font-size",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 12
				}
			}
		}, {
			"property": "line-height",
			"value": {
				"type": "length",
				"value": {
					"unit": "px",
					"value": 16
				}
			}
		}],
		"translate-x-4": [{
			"property": "transform-translate-x",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"kind": {
							"type": "keyword",
							"value": "translateX"
						},
						"value": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 16
							}
						}
					}
				}]
			}
		}],
		"translate-y-4": [{
			"property": "transform-translate-y",
			"value": {
				"type": "list",
				"values": [{
					"type": "record",
					"fields": {
						"kind": {
							"type": "keyword",
							"value": "translateY"
						},
						"value": {
							"type": "length",
							"value": {
								"unit": "px",
								"value": 16
							}
						}
					}
				}]
			}
		}],
		"truncate": [
			{
				"property": "overflow",
				"value": {
					"type": "keyword",
					"value": "hidden"
				}
			},
			{
				"property": "white-space",
				"value": {
					"type": "keyword",
					"value": "nowrap"
				}
			},
			{
				"property": "text-overflow",
				"value": {
					"type": "keyword",
					"value": "ellipsis"
				}
			}
		],
		"whitespace-normal": [{
			"property": "white-space",
			"value": {
				"type": "keyword",
				"value": "normal"
			}
		}],
		"whitespace-nowrap": [{
			"property": "white-space",
			"value": {
				"type": "keyword",
				"value": "nowrap"
			}
		}],
		"z-0": [{
			"property": "z-index",
			"value": {
				"type": "number",
				"value": 0
			}
		}],
		"z-10": [{
			"property": "z-index",
			"value": {
				"type": "number",
				"value": 10
			}
		}],
		"z-20": [{
			"property": "z-index",
			"value": {
				"type": "number",
				"value": 20
			}
		}],
		"z-30": [{
			"property": "z-index",
			"value": {
				"type": "number",
				"value": 30
			}
		}],
		"z-40": [{
			"property": "z-index",
			"value": {
				"type": "number",
				"value": 40
			}
		}],
		"z-50": [{
			"property": "z-index",
			"value": {
				"type": "number",
				"value": 50
			}
		}]
	},
	dynamicRules: [
		{
			"resolver": "spacing",
			"prefixes": [
				{
					"name": "p",
					"properties": [
						"padding-top",
						"padding-right",
						"padding-bottom",
						"padding-left"
					]
				},
				{
					"name": "px",
					"properties": ["padding-left", "padding-right"]
				},
				{
					"name": "py",
					"properties": ["padding-top", "padding-bottom"]
				},
				{
					"name": "pt",
					"properties": ["padding-top"]
				},
				{
					"name": "pr",
					"properties": ["padding-right"]
				},
				{
					"name": "pb",
					"properties": ["padding-bottom"]
				},
				{
					"name": "pl",
					"properties": ["padding-left"]
				},
				{
					"name": "m",
					"properties": [
						"margin-top",
						"margin-right",
						"margin-bottom",
						"margin-left"
					]
				},
				{
					"name": "mx",
					"properties": ["margin-left", "margin-right"]
				},
				{
					"name": "my",
					"properties": ["margin-top", "margin-bottom"]
				},
				{
					"name": "mt",
					"properties": ["margin-top"]
				},
				{
					"name": "mr",
					"properties": ["margin-right"]
				},
				{
					"name": "mb",
					"properties": ["margin-bottom"]
				},
				{
					"name": "ml",
					"properties": ["margin-left"]
				},
				{
					"name": "ms",
					"properties": ["margin-inline-start"]
				},
				{
					"name": "me",
					"properties": ["margin-inline-end"]
				},
				{
					"name": "gap",
					"properties": ["row-gap", "column-gap"]
				},
				{
					"name": "gap-x",
					"properties": ["column-gap"]
				},
				{
					"name": "gap-y",
					"properties": ["row-gap"]
				}
			]
		},
		{
			"resolver": "dimension",
			"prefixes": [
				{
					"name": "w",
					"properties": ["width"]
				},
				{
					"name": "h",
					"properties": ["height"]
				},
				{
					"name": "min-w",
					"properties": ["min-width"]
				},
				{
					"name": "min-h",
					"properties": ["min-height"]
				},
				{
					"name": "max-w",
					"properties": ["max-width"]
				},
				{
					"name": "max-h",
					"properties": ["max-height"]
				},
				{
					"name": "top",
					"properties": ["top"]
				},
				{
					"name": "right",
					"properties": ["right"]
				},
				{
					"name": "bottom",
					"properties": ["bottom"]
				},
				{
					"name": "left",
					"properties": ["left"]
				},
				{
					"name": "inset",
					"properties": [
						"top",
						"right",
						"bottom",
						"left"
					]
				}
			]
		},
		{
			"resolver": "color",
			"prefixes": [
				{
					"name": "bg",
					"properties": ["background-color"]
				},
				{
					"name": "text",
					"properties": ["color"]
				},
				{
					"name": "border",
					"properties": ["border-color"]
				}
			]
		},
		{
			"resolver": "length",
			"prefixes": [
				{
					"name": "rounded",
					"properties": ["border-radius"]
				},
				{
					"name": "text",
					"properties": ["font-size"]
				},
				{
					"name": "border",
					"properties": ["border-width"]
				}
			]
		},
		{
			"resolver": "opacity",
			"prefixes": [{
				"name": "opacity",
				"properties": ["opacity"]
			}]
		},
		{
			"resolver": "number",
			"prefixes": [{
				"name": "z",
				"properties": ["z-index"]
			}]
		},
		{
			"resolver": "ratio",
			"prefixes": [{
				"name": "aspect",
				"properties": ["aspect-ratio"]
			}]
		},
		{
			"resolver": "translate",
			"prefixes": [{
				"name": "translate-x",
				"properties": ["transform-translate-x"]
			}, {
				"name": "translate-y",
				"properties": ["transform-translate-y"]
			}]
		},
		{
			"resolver": "scale",
			"prefixes": [{
				"name": "scale",
				"properties": ["transform-scale"]
			}]
		},
		{
			"resolver": "rotate",
			"prefixes": [{
				"name": "rotate",
				"properties": ["transform-rotate"]
			}]
		}
	],
	conformance: [
		{
			"className": "flex",
			"declarations": [{
				"property": "display",
				"value": {
					"type": "keyword",
					"value": "flex"
				}
			}]
		},
		{
			"className": "flex-1",
			"declarations": [
				{
					"property": "flex-grow",
					"value": {
						"type": "number",
						"value": 1
					}
				},
				{
					"property": "flex-shrink",
					"value": {
						"type": "number",
						"value": 1
					}
				},
				{
					"property": "flex-basis",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": 0
						}
					}
				}
			]
		},
		{
			"className": "px-3",
			"declarations": [{
				"property": "padding-left",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 12
					}
				}
			}, {
				"property": "padding-right",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 12
					}
				}
			}]
		},
		{
			"className": "px-[13px]",
			"declarations": [{
				"property": "padding-left",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 13
					}
				}
			}, {
				"property": "padding-right",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 13
					}
				}
			}]
		},
		{
			"className": "gap-x-4",
			"declarations": [{
				"property": "column-gap",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 16
					}
				}
			}]
		},
		{
			"className": "w-full",
			"declarations": [{
				"property": "width",
				"value": {
					"type": "length",
					"value": {
						"unit": "percent",
						"value": 1
					}
				}
			}]
		},
		{
			"className": "w-2/3",
			"declarations": [{
				"property": "width",
				"value": {
					"type": "length",
					"value": {
						"unit": "percent",
						"value": .6666667
					}
				}
			}]
		},
		{
			"className": "max-w-md",
			"declarations": [{
				"property": "max-width",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 448
					}
				}
			}]
		},
		{
			"className": "w-38%",
			"declarations": [{
				"property": "width",
				"value": {
					"type": "length",
					"value": {
						"unit": "percent",
						"value": .38
					}
				}
			}]
		},
		{
			"className": "min-h-0",
			"declarations": [{
				"property": "min-height",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 0
					}
				}
			}]
		},
		{
			"className": "inset-[5%]",
			"declarations": [
				{
					"property": "top",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": .05
						}
					}
				},
				{
					"property": "right",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": .05
						}
					}
				},
				{
					"property": "bottom",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": .05
						}
					}
				},
				{
					"property": "left",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": .05
						}
					}
				}
			]
		},
		{
			"className": "bg-slate-900",
			"declarations": [{
				"property": "background-color",
				"value": {
					"type": "color",
					"value": {
						"kind": "literal",
						"rgba": 253176575
					}
				}
			}]
		},
		{
			"className": "bg-slate-800/60",
			"declarations": [{
				"property": "background-color",
				"value": {
					"type": "color",
					"value": {
						"kind": "literal",
						"rgba": 506018713
					}
				}
			}]
		},
		{
			"className": "border-red-500",
			"declarations": [{
				"property": "border-color",
				"value": {
					"type": "color",
					"value": {
						"kind": "literal",
						"rgba": 4014228735
					}
				}
			}]
		},
		{
			"className": "opacity-50",
			"declarations": [{
				"property": "opacity",
				"value": {
					"type": "number",
					"value": .5
				}
			}]
		},
		{
			"className": "rounded-xl",
			"declarations": [{
				"property": "border-radius",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 12
					}
				}
			}]
		},
		{
			"className": "text-sm",
			"declarations": [{
				"property": "font-size",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 14
					}
				}
			}, {
				"property": "line-height",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 20
					}
				}
			}]
		},
		{
			"className": "translate-x-4",
			"declarations": [{
				"property": "transform-translate-x",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "translateX"
							},
							"value": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 16
								}
							}
						}
					}]
				}
			}]
		},
		{
			"className": "scale-150",
			"declarations": [{
				"property": "transform-scale",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "scale"
							},
							"value": {
								"type": "list",
								"values": [{
									"type": "number",
									"value": 1.5
								}, {
									"type": "number",
									"value": 1.5
								}]
							}
						}
					}]
				}
			}]
		},
		{
			"className": "rotate-45",
			"declarations": [{
				"property": "transform-rotate",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "rotate"
							},
							"value": {
								"type": "number",
								"value": .7853982
							}
						}
					}]
				}
			}]
		},
		{
			"className": "z-[42]",
			"declarations": [{
				"property": "z-index",
				"value": {
					"type": "number",
					"value": 42
				}
			}]
		},
		{
			"className": "aspect-[16/9]",
			"declarations": [{
				"property": "aspect-ratio",
				"value": {
					"type": "number",
					"value": 1.7777778
				}
			}]
		},
		{
			"className": "-mt-4",
			"declarations": [{
				"property": "margin-top",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": -16
					}
				}
			}]
		},
		{
			"className": "mx-auto",
			"declarations": [{
				"property": "margin-left",
				"value": {
					"type": "length",
					"value": { "unit": "auto" }
				}
			}, {
				"property": "margin-right",
				"value": {
					"type": "length",
					"value": { "unit": "auto" }
				}
			}]
		},
		{
			"className": "-inset-[5%]",
			"declarations": [
				{
					"property": "top",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": -.05
						}
					}
				},
				{
					"property": "right",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": -.05
						}
					}
				},
				{
					"property": "bottom",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": -.05
						}
					}
				},
				{
					"property": "left",
					"value": {
						"type": "length",
						"value": {
							"unit": "percent",
							"value": -.05
						}
					}
				}
			]
		},
		{
			"className": "bg-[#336699cc]",
			"declarations": [{
				"property": "background-color",
				"value": {
					"type": "color",
					"value": {
						"kind": "literal",
						"rgba": 862362060
					}
				}
			}]
		},
		{
			"className": "rounded-[10px]",
			"declarations": [{
				"property": "border-radius",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 10
					}
				}
			}]
		},
		{
			"className": "text-[18px]",
			"declarations": [{
				"property": "font-size",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 18
					}
				}
			}]
		},
		{
			"className": "border-[3px]",
			"declarations": [{
				"property": "border-width",
				"value": {
					"type": "length",
					"value": {
						"unit": "px",
						"value": 3
					}
				}
			}]
		},
		{
			"className": "translate-y-6",
			"declarations": [{
				"property": "transform-translate-y",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "translateY"
							},
							"value": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 24
								}
							}
						}
					}]
				}
			}]
		},
		{
			"className": "-translate-x-2",
			"declarations": [{
				"property": "transform-translate-x",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "translateX"
							},
							"value": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": -8
								}
							}
						}
					}]
				}
			}]
		},
		{
			"className": "scale-125",
			"declarations": [{
				"property": "transform-scale",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "scale"
							},
							"value": {
								"type": "list",
								"values": [{
									"type": "number",
									"value": 1.25
								}, {
									"type": "number",
									"value": 1.25
								}]
							}
						}
					}]
				}
			}]
		},
		{
			"className": "rotate-30",
			"declarations": [{
				"property": "transform-rotate",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "rotate"
							},
							"value": {
								"type": "number",
								"value": .5235988
							}
						}
					}]
				}
			}]
		},
		{
			"className": "-rotate-30",
			"declarations": [{
				"property": "transform-rotate",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"kind": {
								"type": "keyword",
								"value": "rotate"
							},
							"value": {
								"type": "number",
								"value": -.5235988
							}
						}
					}]
				}
			}]
		},
		{
			"className": "grid-cols-3",
			"declarations": [{
				"property": "grid-template-columns",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"count": {
								"type": "number",
								"value": 3
							},
							"kind": {
								"type": "keyword",
								"value": "repeat"
							},
							"values": {
								"type": "list",
								"values": [{
									"type": "record",
									"fields": {
										"kind": {
											"type": "keyword",
											"value": "breadth"
										},
										"value": {
											"type": "record",
											"fields": {
												"kind": {
													"type": "keyword",
													"value": "flex"
												},
												"value": {
													"type": "number",
													"value": 1
												}
											}
										}
									}
								}]
							}
						}
					}]
				}
			}]
		},
		{
			"className": "shadow-md",
			"declarations": [{
				"property": "box-shadow",
				"value": {
					"type": "list",
					"values": [{
						"type": "record",
						"fields": {
							"color": {
								"type": "color",
								"value": {
									"kind": "literal",
									"rgba": 253176346
								}
							},
							"spread": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 0
								}
							},
							"stdDev": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 6
								}
							},
							"x": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 0
								}
							},
							"y": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 3
								}
							}
						}
					}, {
						"type": "record",
						"fields": {
							"color": {
								"type": "color",
								"value": {
									"kind": "literal",
									"rgba": 253176356
								}
							},
							"spread": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": -2
								}
							},
							"stdDev": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 2.5
								}
							},
							"x": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 0
								}
							},
							"y": {
								"type": "length",
								"value": {
									"unit": "px",
									"value": 5
								}
							}
						}
					}]
				}
			}]
		}
	]
};
function matchDynamic(utility, resolver) {
	const negative = [
		"spacing",
		"dimension",
		"translate",
		"rotate"
	].includes(resolver) && utility.startsWith("-");
	const normalized = negative ? utility.slice(1) : utility;
	const rule = wabouUtilityManifest.dynamicRules.find((candidate) => candidate.resolver === resolver);
	for (const prefix of [...rule?.prefixes ?? []].sort((left, right) => right.name.length - left.name.length)) {
		const marker = `${prefix.name}-`;
		if (normalized.startsWith(marker)) return {
			name: prefix.name,
			token: normalized.slice(marker.length),
			properties: prefix.properties,
			negative
		};
	}
}
function parseLength(token, spacing) {
	if (token === "auto") return { unit: "auto" };
	if (token === "full") return {
		unit: "percent",
		value: 1
	};
	if (spacing && token === "px") return {
		unit: "px",
		value: 1
	};
	const scale = spacing ? wabouUtilityManifest.spacing[token] : void 0;
	if (scale !== void 0) return {
		unit: "px",
		value: scale
	};
	if (token.endsWith("%")) {
		const value = Number(token.slice(0, -1));
		if (Number.isFinite(value)) return {
			unit: "percent",
			value: value / 100
		};
	}
	const raw = token.startsWith("[") && token.endsWith("]") ? token.slice(1, -1) : void 0;
	if (!raw) return;
	if (raw.endsWith("px")) {
		const value = Number(raw.slice(0, -2));
		if (Number.isFinite(value)) return {
			unit: "px",
			value
		};
	}
	if (raw.endsWith("rem")) {
		const value = Number(raw.slice(0, -3));
		if (Number.isFinite(value)) return {
			unit: "px",
			value: value * 16
		};
	}
	if (raw.endsWith("%")) {
		const value = Number(raw.slice(0, -1));
		if (Number.isFinite(value)) return {
			unit: "percent",
			value: value / 100
		};
	}
}
function parseDimensionFraction(token) {
	const match = /^(\d+)\/([1-9]\d*)$/.exec(token);
	if (!match) return;
	const numerator = Number(match[1]);
	const denominator = Number(match[2]);
	if (!Number.isInteger(numerator) || !Number.isInteger(denominator) || numerator > 4294967295 || denominator > 4294967295) return;
	return {
		unit: "percent",
		value: Math.fround(numerator / denominator)
	};
}
function parseDimensionLength(token) {
	return parseLength(token, false) ?? parseDimensionFraction(token) ?? parseLength(token, true);
}
function negateLength(value) {
	if (value.unit === "auto") return;
	return {
		...value,
		value: -value.value
	};
}
const rustF32 = Math.fround;
const lengthDeclaration = (property, value) => ({
	property,
	value: {
		type: "length",
		value
	}
});
const transformDeclaration = (kind, value) => ({
	property: {
		translateX: "transform-translate-x",
		translateY: "transform-translate-y",
		scale: "transform-scale",
		rotate: "transform-rotate"
	}[kind] ?? "transform-component",
	value: {
		type: "list",
		values: [{
			type: "record",
			fields: {
				kind: {
					type: "keyword",
					value: kind
				},
				value
			}
		}]
	}
});
function parseCandidate(candidate) {
	const matcher = candidate;
	if (matcher.includes(":")) return {
		candidate,
		message: `Wabou variants are not supported in \`${candidate}\`; use Solid classList or typed style`
	};
	let declarations = wabouUtilityManifest.staticUtilities[matcher];
	const spacing = matchDynamic(matcher, "spacing");
	if (!declarations && spacing) {
		let value = parseLength(spacing.token, true);
		if (value?.unit === "auto" && !spacing.properties.every((property) => property.startsWith("margin-"))) return {
			candidate,
			message: `invalid Wabou spacing in \`${candidate}\`; auto is only valid for margins`
		};
		if (spacing.negative) {
			if (!spacing.properties.every((property) => property.startsWith("margin-"))) return {
				candidate,
				message: `invalid negative Wabou spacing in \`${candidate}\`; only margins may be negative`
			};
			value = value && negateLength(value);
		}
		if (!value) return {
			candidate,
			message: `invalid Wabou spacing in \`${candidate}\`; expected a scale token, px, rem, or percentage`
		};
		declarations = spacing.properties.map((property) => lengthDeclaration(property, value));
	}
	const dimension = matchDynamic(matcher, "dimension");
	if (!declarations && dimension) {
		let value = parseDimensionLength(dimension.token);
		if (dimension.negative) {
			if (!dimension.properties.every((property) => [
				"top",
				"right",
				"bottom",
				"left"
			].includes(property))) return {
				candidate,
				message: `invalid negative Wabou dimension in \`${candidate}\`; only positioned edges may be negative`
			};
			value = value && negateLength(value);
		}
		if (!value) return {
			candidate,
			message: `invalid Wabou dimension in \`${candidate}\`; expected auto, full, a fraction, a scale token, px, rem, or percentage`
		};
		declarations = dimension.properties.map((name) => lengthDeclaration(name, value));
	}
	const lengthRule = matchDynamic(matcher, "length");
	if (!declarations && lengthRule) {
		const value = parseLength(lengthRule.token, false);
		if (value) declarations = lengthRule.properties.map((property) => lengthDeclaration(property, value));
	}
	const color = matchDynamic(matcher, "color");
	if (!declarations && color) {
		const [colorName, opacityToken] = color.token.split("/", 2);
		let rgba = wabouUtilityManifest.colors[colorName];
		const arbitrary = colorName.match(/^\[#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})\]$/)?.[1];
		if (rgba === void 0 && arbitrary) {
			rgba = Number.parseInt(arbitrary, 16);
			if (arbitrary.length === 6) rgba = (rgba << 8 | 255) >>> 0;
		}
		if (rgba !== void 0 && opacityToken !== void 0) {
			const opacity = Number(opacityToken);
			rgba = Number.isFinite(opacity) && opacity >= 0 && opacity <= 100 ? (rgba & 4294967040 | Math.round(opacity * 2.55)) >>> 0 : void 0;
		}
		if (rgba === void 0) return {
			candidate,
			message: `unknown Wabou theme color in \`${candidate}\``
		};
		declarations = [{
			property: color.properties[0],
			value: {
				type: "color",
				value: {
					kind: "literal",
					rgba
				}
			}
		}];
	}
	const opacityRule = matchDynamic(matcher, "opacity");
	if (!declarations && opacityRule) {
		const opacity = Number(opacityRule.token);
		if (!Number.isFinite(opacity) || opacity < 0 || opacity > 100) return {
			candidate,
			message: `invalid Wabou opacity in \`${candidate}\``
		};
		declarations = [{
			property: opacityRule.properties[0],
			value: {
				type: "number",
				value: opacity / 100
			}
		}];
	}
	const numberRule = matchDynamic(matcher, "number");
	if (!declarations && numberRule) {
		const raw = numberRule.token.match(/^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/)?.[1];
		const value = raw === void 0 ? NaN : Number(raw);
		if (!Number.isFinite(value)) return {
			candidate,
			message: `invalid Wabou number in \`${candidate}\`; expected an arbitrary finite number`
		};
		declarations = [{
			property: numberRule.properties[0],
			value: {
				type: "number",
				value
			}
		}];
	}
	const ratioRule = matchDynamic(matcher, "ratio");
	if (!declarations && ratioRule) {
		const raw = ratioRule.token.startsWith("[") && ratioRule.token.endsWith("]") ? ratioRule.token.slice(1, -1) : void 0;
		const parts = raw?.split("/", 2);
		const value = rustF32(parts?.length === 2 ? Number(parts[0]) / Number(parts[1]) : Number(raw));
		if (!Number.isFinite(value) || value <= 0) return {
			candidate,
			message: `invalid Wabou ratio in \`${candidate}\`; expected an arbitrary positive ratio`
		};
		declarations = [{
			property: ratioRule.properties[0],
			value: {
				type: "number",
				value
			}
		}];
	}
	const translateRule = matchDynamic(matcher, "translate");
	if (!declarations && translateRule) {
		let value = parseLength(translateRule.token, true);
		if (translateRule.negative) value = value && negateLength(value);
		if (!value) return {
			candidate,
			message: `invalid Wabou translate in \`${candidate}\``
		};
		declarations = [{
			property: translateRule.properties[0],
			value: {
				type: "list",
				values: [{
					type: "record",
					fields: {
						kind: {
							type: "keyword",
							value: translateRule.name === "translate-x" ? "translateX" : "translateY"
						},
						value: {
							type: "length",
							value
						}
					}
				}]
			}
		}];
	}
	const scaleRule = matchDynamic(matcher, "scale");
	if (!declarations && scaleRule) {
		const arbitrary = scaleRule.token.match(/^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/)?.[1];
		const scale = rustF32(arbitrary === void 0 ? Number(scaleRule.token) / 100 : Number(arbitrary));
		if (!Number.isFinite(scale)) return {
			candidate,
			message: `invalid Wabou scale in \`${candidate}\``
		};
		declarations = [transformDeclaration("scale", {
			type: "list",
			values: [{
				type: "number",
				value: scale
			}, {
				type: "number",
				value: scale
			}]
		})];
	}
	const rotateRule = matchDynamic(matcher, "rotate");
	if (!declarations && rotateRule) {
		const arbitrary = rotateRule.token.match(/^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/)?.[1];
		let degrees = Number(arbitrary ?? rotateRule.token);
		if (rotateRule.negative) degrees = -degrees;
		const radians = rustF32(degrees * Math.PI / 180);
		if (!Number.isFinite(radians)) return {
			candidate,
			message: `invalid Wabou rotation in \`${candidate}\``
		};
		declarations = [transformDeclaration("rotate", {
			type: "number",
			value: radians
		})];
	}
	if (!declarations) return {
		candidate,
		message: `unsupported Wabou utility \`${candidate}\``
	};
	return {
		candidate,
		matcher,
		declarations
	};
}
function resolveWabouUtility(candidate) {
	const result = parseCandidate(candidate);
	return "declarations" in result ? result : void 0;
}
function validateWabouUtility(candidate) {
	const result = parseCandidate(candidate);
	return "message" in result ? result : void 0;
}
function cssValue(value) {
	switch (value.type) {
		case "keyword": return value.value;
		case "boolean": return String(value.value);
		case "number": return value.value;
		case "length":
			if (value.value.unit === "auto") return "auto";
			return `${value.value.unit === "percent" ? value.value.value * 100 : value.value.value}${value.value.unit === "percent" ? "%" : "px"}`;
		case "color": return `#${value.value.rgba.toString(16).padStart(8, "0")}`;
		case "list": return value.values.map((item) => {
			if (item.type !== "record") return cssValue(item);
			const kind = item.fields.kind;
			const argument = item.fields.value;
			if (kind?.type !== "keyword") return "";
			if (kind.value === "repeat") {
				const count = item.fields.count;
				const tracks = item.fields.values;
				if (count?.type !== "number" || tracks?.type !== "list") return "";
				return `repeat(${count.value}, ${tracks.values.map(cssValue).join(" ")})`;
			}
			if (!argument) return "";
			if (kind.value === "breadth") return cssValue(argument);
			if (kind.value === "flex") return `${cssValue(argument)}fr`;
			const text = argument.type === "list" ? argument.values.map(cssValue).join(", ") : cssValue(argument);
			return `${kind.value}(${text})`;
		}).join(" ");
		case "record": {
			const kind = value.fields.kind;
			const argument = value.fields.value;
			if (kind?.type !== "keyword" || !argument) return "";
			if (kind.value === "breadth") return cssValue(argument);
			if (kind.value === "flex") return `${cssValue(argument)}fr`;
			return "";
		}
	}
}
function unoRule() {
	return [/^.+$/, ([candidate]) => {
		const resolved = resolveWabouUtility(candidate);
		if (!resolved) return;
		return Object.fromEntries(resolved.declarations.map(({ property, value }) => [property.startsWith("transform-") ? "transform" : property, cssValue(value)]));
	}];
}
/** UnoCSS adapter for editor tooling over the native utility manifest. */
function presetWabou() {
	return {
		name: "@wabou/unocss-preset",
		rules: [unoRule()],
		autocomplete: { templates: [
			"p-$spacing",
			"px-$spacing",
			"py-$spacing",
			"m-$spacing",
			"gap-$spacing",
			"w-$spacing",
			"h-$spacing",
			"bg-$colors",
			"text-$colors",
			"border-$colors"
		] }
	};
}
//#endregion
export { presetWabou, resolveWabouUtility, validateWabouUtility, wabouUtilityManifest };

//# sourceMappingURL=index.mjs.map