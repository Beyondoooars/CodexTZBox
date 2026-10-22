// 常用 IANA 时区清单：用于「目标时区」的搜索与选择。
// label 为中文显示名，id 为规范 IANA 标识；搜索同时匹配两者。
// 未收录的时区仍可直接输入合法 IANA 名称（前端与后端都会做校验）。

export type ZoneOption = {
  id: string
  label: string
}

export const ZONE_OPTIONS: ZoneOption[] = [
  // UTC
  { id: 'UTC', label: '协调世界时（UTC）' },

  // 亚洲
  { id: 'Asia/Shanghai', label: '中国标准时间（上海 / 北京）' },
  { id: 'Asia/Urumqi', label: '乌鲁木齐时间' },
  { id: 'Asia/Hong_Kong', label: '香港' },
  { id: 'Asia/Macau', label: '澳门' },
  { id: 'Asia/Taipei', label: '台北' },
  { id: 'Asia/Tokyo', label: '东京' },
  { id: 'Asia/Seoul', label: '首尔' },
  { id: 'Asia/Pyongyang', label: '平壤' },
  { id: 'Asia/Singapore', label: '新加坡' },
  { id: 'Asia/Kuala_Lumpur', label: '吉隆坡' },
  { id: 'Asia/Bangkok', label: '曼谷' },
  { id: 'Asia/Jakarta', label: '雅加达' },
  { id: 'Asia/Makassar', label: '望加锡' },
  { id: 'Asia/Jayapura', label: '查亚普拉' },
  { id: 'Asia/Manila', label: '马尼拉' },
  { id: 'Asia/Ho_Chi_Minh', label: '胡志明市' },
  { id: 'Asia/Phnom_Penh', label: '金边' },
  { id: 'Asia/Vientiane', label: '万象' },
  { id: 'Asia/Yangon', label: '仰光' },
  { id: 'Asia/Dhaka', label: '达卡' },
  { id: 'Asia/Kolkata', label: '印度标准时间（加尔各答 / 新德里）' },
  { id: 'Asia/Kathmandu', label: '加德满都' },
  { id: 'Asia/Colombo', label: '科伦坡' },
  { id: 'Asia/Karachi', label: '卡拉奇' },
  { id: 'Asia/Tashkent', label: '塔什干' },
  { id: 'Asia/Almaty', label: '阿拉木图' },
  { id: 'Asia/Bishkek', label: '比什凯克' },
  { id: 'Asia/Dubai', label: '迪拜' },
  { id: 'Asia/Riyadh', label: '利雅得' },
  { id: 'Asia/Tehran', label: '德黑兰' },
  { id: 'Asia/Baghdad', label: '巴格达' },
  { id: 'Asia/Jerusalem', label: '耶路撒冷' },
  { id: 'Asia/Beirut', label: '贝鲁特' },
  { id: 'Europe/Istanbul', label: '伊斯坦布尔' },
  { id: 'Asia/Yerevan', label: '埃里温' },
  { id: 'Asia/Tbilisi', label: '第比利斯' },
  { id: 'Asia/Baku', label: '巴库' },
  { id: 'Asia/Kabul', label: '喀布尔' },
  { id: 'Asia/Novosibirsk', label: '新西伯利亚' },
  { id: 'Asia/Krasnoyarsk', label: '克拉斯诺亚尔斯克' },
  { id: 'Asia/Irkutsk', label: '伊尔库茨克' },
  { id: 'Asia/Yakutsk', label: '雅库茨克' },
  { id: 'Asia/Vladivostok', label: '符拉迪沃斯托克' },
  { id: 'Asia/Magadan', label: '马加丹' },
  { id: 'Asia/Kamchatka', label: '堪察加' },
  { id: 'Asia/Ulaanbaatar', label: '乌兰巴托' },

  // 欧洲
  { id: 'Europe/London', label: '伦敦' },
  { id: 'Europe/Dublin', label: '都柏林' },
  { id: 'Europe/Lisbon', label: '里斯本' },
  { id: 'Europe/Madrid', label: '马德里' },
  { id: 'Europe/Paris', label: '巴黎' },
  { id: 'Europe/Amsterdam', label: '阿姆斯特丹' },
  { id: 'Europe/Brussels', label: '布鲁塞尔' },
  { id: 'Europe/Berlin', label: '柏林' },
  { id: 'Europe/Zurich', label: '苏黎世' },
  { id: 'Europe/Vienna', label: '维也纳' },
  { id: 'Europe/Prague', label: '布拉格' },
  { id: 'Europe/Warsaw', label: '华沙' },
  { id: 'Europe/Budapest', label: '布达佩斯' },
  { id: 'Europe/Rome', label: '罗马' },
  { id: 'Europe/Athens', label: '雅典' },
  { id: 'Europe/Helsinki', label: '赫尔辛基' },
  { id: 'Europe/Stockholm', label: '斯德哥尔摩' },
  { id: 'Europe/Oslo', label: '奥斯陆' },
  { id: 'Europe/Copenhagen', label: '哥本哈根' },
  { id: 'Europe/Tallinn', label: '塔林' },
  { id: 'Europe/Riga', label: '里加' },
  { id: 'Europe/Vilnius', label: '维尔纽斯' },
  { id: 'Europe/Kyiv', label: '基辅' },
  { id: 'Europe/Minsk', label: '明斯克' },
  { id: 'Europe/Moscow', label: '莫斯科' },
  { id: 'Europe/Bucharest', label: '布加勒斯特' },
  { id: 'Europe/Sofia', label: '索非亚' },
  { id: 'Europe/Belgrade', label: '贝尔格莱德' },
  { id: 'Europe/Zagreb', label: '萨格勒布' },
  { id: 'Atlantic/Reykjavik', label: '雷克雅未克' },

  // 北美
  { id: 'America/Los_Angeles', label: '太平洋时间（洛杉矶）' },
  { id: 'America/Denver', label: '山地时间（丹佛）' },
  { id: 'America/Phoenix', label: '亚利桑那时间（菲尼克斯）' },
  { id: 'America/Chicago', label: '美国中部时间（芝加哥）' },
  { id: 'America/New_York', label: '美国东部时间（纽约）' },
  { id: 'America/Anchorage', label: '阿拉斯加时间（安克雷奇）' },
  { id: 'America/Adak', label: '阿留申群岛' },
  { id: 'Pacific/Honolulu', label: '夏威夷时间（檀香山）' },
  { id: 'America/St_Johns', label: '纽芬兰时间（圣约翰斯）' },
  { id: 'America/Halifax', label: '大西洋时间（哈利法克斯）' },
  { id: 'America/Toronto', label: '多伦多' },
  { id: 'America/Vancouver', label: '温哥华' },
  { id: 'America/Edmonton', label: '埃德蒙顿' },
  { id: 'America/Winnipeg', label: '温尼伯' },
  { id: 'America/Regina', label: '里贾纳' },
  { id: 'America/Whitehorse', label: '白马市' },
  { id: 'America/Mexico_City', label: '墨西哥城' },
  { id: 'America/Tijuana', label: '蒂华纳' },

  // 中南美
  { id: 'America/Guatemala', label: '危地马拉城' },
  { id: 'America/Costa_Rica', label: '圣何塞' },
  { id: 'America/Panama', label: '巴拿马城' },
  { id: 'America/Havana', label: '哈瓦那' },
  { id: 'America/Jamaica', label: '金斯敦' },
  { id: 'America/Santo_Domingo', label: '圣多明各' },
  { id: 'America/Caracas', label: '加拉加斯' },
  { id: 'America/Bogota', label: '波哥大' },
  { id: 'America/Lima', label: '利马' },
  { id: 'America/La_Paz', label: '拉巴斯' },
  { id: 'America/Santiago', label: '圣地亚哥（智利）' },
  { id: 'America/Argentina/Buenos_Aires', label: '布宜诺斯艾利斯' },
  { id: 'America/Montevideo', label: '蒙得维的亚' },
  { id: 'America/Asuncion', label: '亚松森' },
  { id: 'America/Sao_Paulo', label: '圣保罗' },
  { id: 'America/Manaus', label: '马瑙斯' },
  { id: 'America/Recife', label: '累西腓' },
  { id: 'America/Fortaleza', label: '福塔莱萨' },

  // 大洋洲
  { id: 'Australia/Sydney', label: '悉尼' },
  { id: 'Australia/Melbourne', label: '墨尔本' },
  { id: 'Australia/Brisbane', label: '布里斯班' },
  { id: 'Australia/Adelaide', label: '阿德莱德' },
  { id: 'Australia/Perth', label: '珀斯' },
  { id: 'Australia/Darwin', label: '达尔文' },
  { id: 'Australia/Hobart', label: '霍巴特' },
  { id: 'Pacific/Auckland', label: '奥克兰' },
  { id: 'Pacific/Fiji', label: '斐济（苏瓦）' },
  { id: 'Pacific/Noumea', label: '努美阿' },
  { id: 'Pacific/Port_Moresby', label: '莫尔兹比港' },
  { id: 'Pacific/Guam', label: '关岛' },
  { id: 'Pacific/Tahiti', label: '塔希提' },
  { id: 'Pacific/Apia', label: '阿皮亚' },
  { id: 'Pacific/Kiritimati', label: '圣诞岛（UTC+14）' },

  // 非洲
  { id: 'Africa/Cairo', label: '开罗' },
  { id: 'Africa/Casablanca', label: '卡萨布兰卡' },
  { id: 'Africa/Algiers', label: '阿尔及尔' },
  { id: 'Africa/Tunis', label: '突尼斯' },
  { id: 'Africa/Tripoli', label: '的黎波里' },
  { id: 'Africa/Khartoum', label: '喀土穆' },
  { id: 'Africa/Addis_Ababa', label: '亚的斯亚贝巴' },
  { id: 'Africa/Nairobi', label: '内罗毕' },
  { id: 'Africa/Lagos', label: '拉各斯' },
  { id: 'Africa/Accra', label: '阿克拉' },
  { id: 'Africa/Dakar', label: '达喀尔' },
  { id: 'Africa/Kinshasa', label: '金沙萨' },
  { id: 'Africa/Luanda', label: '罗安达' },
  { id: 'Africa/Johannesburg', label: '约翰内斯堡' },
  { id: 'Africa/Maputo', label: '马普托' },
  { id: 'Africa/Harare', label: '哈拉雷' },
]

const ZONE_LABELS = new Map(ZONE_OPTIONS.map((option) => [option.id, option.label]))

function normalize(value: string) {
  return value.toLowerCase().replace(/[\s/_\-.]+/g, '')
}

export function zoneLabel(id: string): string | undefined {
  return ZONE_LABELS.get(id)
}

export function searchZones(query: string): ZoneOption[] {
  const raw = query.trim()
  if (!raw) return ZONE_OPTIONS
  const normalized = normalize(raw)
  const lower = raw.toLowerCase()
  return ZONE_OPTIONS.filter((option) =>
    normalize(option.id).includes(normalized) || option.label.toLowerCase().includes(lower),
  )
}
