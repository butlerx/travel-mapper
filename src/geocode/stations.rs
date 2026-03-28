//! UK CRS station code lookup — maps three-letter CRS codes to station names.

pub(crate) struct Station {
    pub(crate) code: &'static str,
    pub(crate) name: &'static str,
}

static STATIONS: &[Station] = &[
    Station {
        code: "AAP",
        name: "Alexandra Palace",
    },
    Station {
        code: "AAT",
        name: "Achanalt",
    },
    Station {
        code: "ABA",
        name: "Aberdare",
    },
    Station {
        code: "ABC",
        name: "Altnabreac",
    },
    Station {
        code: "ABD",
        name: "Aberdeen",
    },
    Station {
        code: "ABE",
        name: "Aber",
    },
    Station {
        code: "ABF",
        name: "Ashurst (Bald Faced Stag)",
    },
    Station {
        code: "ABH",
        name: "Abererch",
    },
    Station {
        code: "ABW",
        name: "Abbey Wood",
    },
    Station {
        code: "ABY",
        name: "Ashburys",
    },
    Station {
        code: "ACB",
        name: "Acton Bridge",
    },
    Station {
        code: "ACC",
        name: "Acton Central",
    },
    Station {
        code: "ACG",
        name: "Acocks Green",
    },
    Station {
        code: "ACH",
        name: "Achnashellach",
    },
    Station {
        code: "ACK",
        name: "Acklington",
    },
    Station {
        code: "ACL",
        name: "Acle",
    },
    Station {
        code: "ACN",
        name: "Achnasheen",
    },
    Station {
        code: "ACR",
        name: "Accrington",
    },
    Station {
        code: "ACT",
        name: "Ascot (Berks)",
    },
    Station {
        code: "ACY",
        name: "Abercynon",
    },
    Station {
        code: "ADC",
        name: "Adlington (Cheshire)",
    },
    Station {
        code: "ADD",
        name: "Adderley Park",
    },
    Station {
        code: "ADK",
        name: "Ardwick",
    },
    Station {
        code: "ADL",
        name: "Adlington (Lancs)",
    },
    Station {
        code: "ADM",
        name: "Adisham",
    },
    Station {
        code: "ADN",
        name: "Ardrossan Town",
    },
    Station {
        code: "ADR",
        name: "Airdrie",
    },
    Station {
        code: "ADS",
        name: "Ardrossan Harbour",
    },
    Station {
        code: "ADV",
        name: "Andover",
    },
    Station {
        code: "ADW",
        name: "Addiewell",
    },
    Station {
        code: "AER",
        name: "Aberaeron (Bus)",
    },
    Station {
        code: "AFK",
        name: "Ashford International",
    },
    Station {
        code: "AFS",
        name: "Ashford (Surrey)",
    },
    Station {
        code: "AFV",
        name: "Ansdell & Fairhaven",
    },
    Station {
        code: "AGL",
        name: "Abergele & Pensarn",
    },
    Station {
        code: "AGS",
        name: "Argyle Street",
    },
    Station {
        code: "AGT",
        name: "Aldrington",
    },
    Station {
        code: "AGV",
        name: "Abergavenny",
    },
    Station {
        code: "AHD",
        name: "Ashtead",
    },
    Station {
        code: "AHN",
        name: "Ashton-under-Lyne",
    },
    Station {
        code: "AHS",
        name: "Ashurst (Kent)",
    },
    Station {
        code: "AHT",
        name: "Aldershot",
    },
    Station {
        code: "AHV",
        name: "Ash Vale",
    },
    Station {
        code: "AIG",
        name: "Aigburth",
    },
    Station {
        code: "AIN",
        name: "Aintree",
    },
    Station {
        code: "AIR",
        name: "Airbles",
    },
    Station {
        code: "ALB",
        name: "Albrighton",
    },
    Station {
        code: "ALD",
        name: "Alderley Edge",
    },
    Station {
        code: "ALF",
        name: "Alfreton",
    },
    Station {
        code: "ALG",
        name: "Aldeburgh (via Saxmundham)",
    },
    Station {
        code: "ALK",
        name: "Aslockton",
    },
    Station {
        code: "ALM",
        name: "Alnmouth",
    },
    Station {
        code: "ALN",
        name: "Althorne (Essex)",
    },
    Station {
        code: "ALO",
        name: "Alloa",
    },
    Station {
        code: "ALP",
        name: "Althorpe (Humberside)",
    },
    Station {
        code: "ALR",
        name: "Alresford (Essex)",
    },
    Station {
        code: "ALT",
        name: "Altrincham",
    },
    Station {
        code: "ALV",
        name: "Alvechurch",
    },
    Station {
        code: "ALW",
        name: "Allens West",
    },
    Station {
        code: "ALX",
        name: "Alexandria",
    },
    Station {
        code: "AMB",
        name: "Ambergate",
    },
    Station {
        code: "AMF",
        name: "Ammanford",
    },
    Station {
        code: "AML",
        name: "Acton Main Line",
    },
    Station {
        code: "AMM",
        name: "Abraham Moss (Metrolink)",
    },
    Station {
        code: "AMO",
        name: "Ashton Moss (Metrolink)",
    },
    Station {
        code: "AMR",
        name: "Amersham",
    },
    Station {
        code: "AMT",
        name: "Aldermaston",
    },
    Station {
        code: "AMY",
        name: "Amberley",
    },
    Station {
        code: "ANC",
        name: "Ancaster",
    },
    Station {
        code: "AND",
        name: "Anderston",
    },
    Station {
        code: "ANF",
        name: "Ashurst (New Forest)",
    },
    Station {
        code: "ANG",
        name: "Angmering",
    },
    Station {
        code: "ANH",
        name: "Anchorage (Metrolink)",
    },
    Station {
        code: "ANL",
        name: "Anniesland",
    },
    Station {
        code: "ANM",
        name: "Antrim (N Ireland)",
    },
    Station {
        code: "ANN",
        name: "Annan",
    },
    Station {
        code: "ANS",
        name: "Ainsdale",
    },
    Station {
        code: "ANZ",
        name: "Anerley",
    },
    Station {
        code: "AON",
        name: "Alton",
    },
    Station {
        code: "APB",
        name: "Appley Bridge",
    },
    Station {
        code: "APD",
        name: "Appledore (Kent)",
    },
    Station {
        code: "APF",
        name: "Appleford",
    },
    Station {
        code: "APG",
        name: "Aspley Guise",
    },
    Station {
        code: "APN",
        name: "Newcastle Airport",
    },
    Station {
        code: "APP",
        name: "Appleby",
    },
    Station {
        code: "APS",
        name: "Apsley",
    },
    Station {
        code: "APY",
        name: "Apperley Bridge",
    },
    Station {
        code: "ARA",
        name: "Skye (Armadale)",
    },
    Station {
        code: "ARB",
        name: "Arbroath",
    },
    Station {
        code: "ARD",
        name: "Ardgay",
    },
    Station {
        code: "ARG",
        name: "Arisaig",
    },
    Station {
        code: "ARL",
        name: "Arlesey",
    },
    Station {
        code: "ARM",
        name: "Armadale",
    },
    Station {
        code: "ARN",
        name: "Arnside",
    },
    Station {
        code: "ARP",
        name: "Alvcrlp",
    },
    Station {
        code: "ARR",
        name: "Arram",
    },
    Station {
        code: "ART",
        name: "Arrochar & Tarbet",
    },
    Station {
        code: "ARU",
        name: "Arundel",
    },
    Station {
        code: "ARW",
        name: "Arklow      (CIV)",
    },
    Station {
        code: "ASB",
        name: "Ardrossan South Beach",
    },
    Station {
        code: "ASC",
        name: "Ashchurch",
    },
    Station {
        code: "ASD",
        name: "ASHLEY DOWN",
    },
    Station {
        code: "ASF",
        name: "Ashfield",
    },
    Station {
        code: "ASG",
        name: "Alsager",
    },
    Station {
        code: "ASH",
        name: "Ash",
    },
    Station {
        code: "ASI",
        name: "Ashford International (CIV)",
    },
    Station {
        code: "ASK",
        name: "Askam",
    },
    Station {
        code: "ASL",
        name: "ASHINGTON",
    },
    Station {
        code: "ASM",
        name: "Audenshaw (Metrolink)",
    },
    Station {
        code: "ASN",
        name: "Addlestone",
    },
    Station {
        code: "ASP",
        name: "Aspatria",
    },
    Station {
        code: "ASS",
        name: "Alness",
    },
    Station {
        code: "AST",
        name: "Aston",
    },
    Station {
        code: "ASY",
        name: "Ashley",
    },
    Station {
        code: "ATB",
        name: "Attenborough",
    },
    Station {
        code: "ATH",
        name: "Atherstone (Warks)",
    },
    Station {
        code: "ATL",
        name: "Attleborough",
    },
    Station {
        code: "ATM",
        name: "Attymon",
    },
    Station {
        code: "ATN",
        name: "Atherton (Manchester)",
    },
    Station {
        code: "ATO",
        name: "Athlone     (CIV)",
    },
    Station {
        code: "ATR",
        name: "Athenry     (CIV)",
    },
    Station {
        code: "ATT",
        name: "Attadale",
    },
    Station {
        code: "ATY",
        name: "Athy        (CIV)",
    },
    Station {
        code: "AUD",
        name: "Audley End",
    },
    Station {
        code: "AUG",
        name: "Aughton Park",
    },
    Station {
        code: "AUI",
        name: "Ardlui",
    },
    Station {
        code: "AUK",
        name: "Auchinleck",
    },
    Station {
        code: "AUL",
        name: "Ashton-under-Lyne (Metrolink)",
    },
    Station {
        code: "AUR",
        name: "Aberdour",
    },
    Station {
        code: "AUW",
        name: "Ascott-under-Wychwood",
    },
    Station {
        code: "AVF",
        name: "Avoncliff",
    },
    Station {
        code: "AVM",
        name: "Aviemore",
    },
    Station {
        code: "AVN",
        name: "Avonmouth",
    },
    Station {
        code: "AVP",
        name: "Aylesbury Vale Parkway",
    },
    Station {
        code: "AVY",
        name: "Aberdovey",
    },
    Station {
        code: "AWK",
        name: "Adwick",
    },
    Station {
        code: "AWL",
        name: "Ashton West (Metrolink)",
    },
    Station {
        code: "AWM",
        name: "Ashwell & Morden",
    },
    Station {
        code: "AWT",
        name: "Armathwaite",
    },
    Station {
        code: "AXM",
        name: "Axminster",
    },
    Station {
        code: "AXP",
        name: "Alexandra Parade",
    },
    Station {
        code: "AYH",
        name: "Aylesham",
    },
    Station {
        code: "AYL",
        name: "Aylesford",
    },
    Station {
        code: "AYP",
        name: "Albany Park",
    },
    Station {
        code: "AYR",
        name: "Ayr",
    },
    Station {
        code: "AYS",
        name: "Aylesbury",
    },
    Station {
        code: "AYW",
        name: "Aberystwyth",
    },
    Station {
        code: "BAA",
        name: "Barnham",
    },
    Station {
        code: "BAB",
        name: "Balcombe",
    },
    Station {
        code: "BAC",
        name: "Bache",
    },
    Station {
        code: "BAD",
        name: "Banstead",
    },
    Station {
        code: "BAG",
        name: "Bagshot",
    },
    Station {
        code: "BAH",
        name: "Bank Hall",
    },
    Station {
        code: "BAI",
        name: "Blairhill",
    },
    Station {
        code: "BAJ",
        name: "Baglan",
    },
    Station {
        code: "BAK",
        name: "Battersea Park",
    },
    Station {
        code: "BAL",
        name: "Balham",
    },
    Station {
        code: "BAM",
        name: "Bamford",
    },
    Station {
        code: "BAN",
        name: "Banbury",
    },
    Station {
        code: "BAO",
        name: "Ballymote   (CIV)",
    },
    Station {
        code: "BAR",
        name: "Bare Lane",
    },
    Station {
        code: "BAS",
        name: "Bere Alston",
    },
    Station {
        code: "BAT",
        name: "Battle",
    },
    Station {
        code: "BAU",
        name: "Barton-on-Humber",
    },
    Station {
        code: "BAV",
        name: "Barrow Haven",
    },
    Station {
        code: "BAW",
        name: "Blackwater",
    },
    Station {
        code: "BAX",
        name: "Ballina     (CIV)",
    },
    Station {
        code: "BAY",
        name: "Bayford",
    },
    Station {
        code: "BBG",
        name: "Bishopbriggs",
    },
    Station {
        code: "BBK",
        name: "Bilbrook",
    },
    Station {
        code: "BBL",
        name: "Bat & Ball",
    },
    Station {
        code: "BBN",
        name: "Blackburn",
    },
    Station {
        code: "BBS",
        name: "Bordesley",
    },
    Station {
        code: "BBW",
        name: "Berry Brow",
    },
    Station {
        code: "BBY",
        name: "Ballybrophy (CIV)",
    },
    Station {
        code: "BCB",
        name: "Burscough Bridge",
    },
    Station {
        code: "BCC",
        name: "Beccles",
    },
    Station {
        code: "BCE",
        name: "Bracknell",
    },
    Station {
        code: "BCF",
        name: "Beaconsfield",
    },
    Station {
        code: "BCG",
        name: "Birchgrove",
    },
    Station {
        code: "BCH",
        name: "Birchington",
    },
    Station {
        code: "BCJ",
        name: "Burscough Junction",
    },
    Station {
        code: "BCK",
        name: "Buckley",
    },
    Station {
        code: "BCN",
        name: "Branchton",
    },
    Station {
        code: "BCS",
        name: "Bicester North",
    },
    Station {
        code: "BCU",
        name: "Brockenhurst",
    },
    Station {
        code: "BCV",
        name: "Bruce Grove",
    },
    Station {
        code: "BCY",
        name: "Brockley",
    },
    Station {
        code: "BCZ",
        name: "Brent Cross West",
    },
    Station {
        code: "BDA",
        name: "Brundall",
    },
    Station {
        code: "BDB",
        name: "Broadbottom",
    },
    Station {
        code: "BDC",
        name: "Brodick",
    },
    Station {
        code: "BDF",
        name: "Bodmin Mount FLL",
    },
    Station {
        code: "BDG",
        name: "Bridgeton",
    },
    Station {
        code: "BDH",
        name: "Bedhampton",
    },
    Station {
        code: "BDI",
        name: "Bradford Interchange",
    },
    Station {
        code: "BDK",
        name: "Baldock",
    },
    Station {
        code: "BDL",
        name: "Birkdale",
    },
    Station {
        code: "BDM",
        name: "Bedford",
    },
    Station {
        code: "BDN",
        name: "Brading",
    },
    Station {
        code: "BDQ",
        name: "Bradford Forster Square",
    },
    Station {
        code: "BDS",
        name: "Bond Street (Elizabeth line)",
    },
    Station {
        code: "BDT",
        name: "Bridlington",
    },
    Station {
        code: "BDW",
        name: "Bedwyn",
    },
    Station {
        code: "BDY",
        name: "Bredbury",
    },
    Station {
        code: "BDZ",
        name: "Bordon",
    },
    Station {
        code: "BEA",
        name: "Bridge of Allan",
    },
    Station {
        code: "BEB",
        name: "Bebington",
    },
    Station {
        code: "BEC",
        name: "Beckenham Hill",
    },
    Station {
        code: "BEE",
        name: "Beeston",
    },
    Station {
        code: "BEF",
        name: "Benfleet",
    },
    Station {
        code: "BEG",
        name: "Beltring",
    },
    Station {
        code: "BEH",
        name: "Bedworth",
    },
    Station {
        code: "BEJ",
        name: "BEDLINGTON",
    },
    Station {
        code: "BEL",
        name: "Beauly",
    },
    Station {
        code: "BEM",
        name: "Bempton",
    },
    Station {
        code: "BEN",
        name: "Bentham",
    },
    Station {
        code: "BEP",
        name: "Bermuda Park",
    },
    Station {
        code: "BER",
        name: "Bearley",
    },
    Station {
        code: "BES",
        name: "Bescar Lane",
    },
    Station {
        code: "BET",
        name: "Bethnal Green",
    },
    Station {
        code: "BEU",
        name: "Beaulieu Road",
    },
    Station {
        code: "BEV",
        name: "Beverley",
    },
    Station {
        code: "BEX",
        name: "Bexhill",
    },
    Station {
        code: "BEY",
        name: "Ben Rhydding",
    },
    Station {
        code: "BFA",
        name: "Belfast Port",
    },
    Station {
        code: "BFB",
        name: "Belford Bus",
    },
    Station {
        code: "BFC",
        name: "Lanyon Place (Belfast)",
    },
    Station {
        code: "BFD",
        name: "Brentford",
    },
    Station {
        code: "BFE",
        name: "Bere Ferrers",
    },
    Station {
        code: "BFF",
        name: "Blaenau Ffestiniog",
    },
    Station {
        code: "BFN",
        name: "Byfleet & New Haw",
    },
    Station {
        code: "BFR",
        name: "London Blackfriars",
    },
    Station {
        code: "BGA",
        name: "Brundall Gardens",
    },
    Station {
        code: "BGD",
        name: "Bargoed",
    },
    Station {
        code: "BGE",
        name: "Broad Green",
    },
    Station {
        code: "BGG",
        name: "Brigg",
    },
    Station {
        code: "BGH",
        name: "Brighouse",
    },
    Station {
        code: "BGI",
        name: "Bargeddie",
    },
    Station {
        code: "BGK",
        name: "Baguley (Metrolink)",
    },
    Station {
        code: "BGL",
        name: "Bugle",
    },
    Station {
        code: "BGM",
        name: "Bellingham (London)",
    },
    Station {
        code: "BGN",
        name: "Bridgend",
    },
    Station {
        code: "BGS",
        name: "Bogston",
    },
    Station {
        code: "BGV",
        name: "Barking Riverside",
    },
    Station {
        code: "BHA",
        name: "BOURNMTH AIR BUS",
    },
    Station {
        code: "BHC",
        name: "Balloch Central",
    },
    Station {
        code: "BHD",
        name: "Brithdir",
    },
    Station {
        code: "BHG",
        name: "Bathgate",
    },
    Station {
        code: "BHI",
        name: "Birmingham International",
    },
    Station {
        code: "BHK",
        name: "Bush Hill Park",
    },
    Station {
        code: "BHM",
        name: "Birmingham New Street",
    },
    Station {
        code: "BHN",
        name: "Ballyhaunis (CIV)",
    },
    Station {
        code: "BHO",
        name: "Blackhorse Road",
    },
    Station {
        code: "BHP",
        name: "Blenheim Palace",
    },
    Station {
        code: "BHR",
        name: "Builth Road",
    },
    Station {
        code: "BHS",
        name: "Brockholes",
    },
    Station {
        code: "BIA",
        name: "Bishop Auckland",
    },
    Station {
        code: "BIC",
        name: "Billericay",
    },
    Station {
        code: "BID",
        name: "Bidston",
    },
    Station {
        code: "BIF",
        name: "Barrow in Furness",
    },
    Station {
        code: "BIG",
        name: "Billingshurst",
    },
    Station {
        code: "BIH",
        name: "Birdhill    (CIV)",
    },
    Station {
        code: "BIK",
        name: "Birkbeck",
    },
    Station {
        code: "BIL",
        name: "Billingham (Cleveland)",
    },
    Station {
        code: "BIN",
        name: "Bingham",
    },
    Station {
        code: "BIO",
        name: "Baillieston",
    },
    Station {
        code: "BIP",
        name: "Bishopstone (Sussex)",
    },
    Station {
        code: "BIS",
        name: "Bishops Stortford",
    },
    Station {
        code: "BIT",
        name: "Bicester Village",
    },
    Station {
        code: "BIW",
        name: "Biggleswade",
    },
    Station {
        code: "BIY",
        name: "Bingley",
    },
    Station {
        code: "BKA",
        name: "Bookham",
    },
    Station {
        code: "BKC",
        name: "Birkenhead Central",
    },
    Station {
        code: "BKD",
        name: "Blakedown",
    },
    Station {
        code: "BKG",
        name: "Barking",
    },
    Station {
        code: "BKH",
        name: "Blackheath",
    },
    Station {
        code: "BKI",
        name: "Birkenhead 12 Quay",
    },
    Station {
        code: "BKJ",
        name: "Beckenham Junction",
    },
    Station {
        code: "BKL",
        name: "Bickley",
    },
    Station {
        code: "BKM",
        name: "Berkhamsted",
    },
    Station {
        code: "BKN",
        name: "Birkenhead North",
    },
    Station {
        code: "BKO",
        name: "Brookwood",
    },
    Station {
        code: "BKP",
        name: "Birkenhead Park",
    },
    Station {
        code: "BKQ",
        name: "Birkenhead Hamilton Square",
    },
    Station {
        code: "BKR",
        name: "Blackridge",
    },
    Station {
        code: "BKS",
        name: "Bekesbourne",
    },
    Station {
        code: "BKT",
        name: "Blake Street",
    },
    Station {
        code: "BKV",
        name: "Bowker Vale (Metrolink)",
    },
    Station {
        code: "BKW",
        name: "Berkswell",
    },
    Station {
        code: "BLA",
        name: "Blair Atholl",
    },
    Station {
        code: "BLB",
        name: "Battlesbridge",
    },
    Station {
        code: "BLD",
        name: "Baildon",
    },
    Station {
        code: "BLE",
        name: "Bramley (West Yorks)",
    },
    Station {
        code: "BLG",
        name: "Bellgrove",
    },
    Station {
        code: "BLH",
        name: "Bellshill",
    },
    Station {
        code: "BLI",
        name: "BLYTH BEBSIDE",
    },
    Station {
        code: "BLK",
        name: "Blackrod",
    },
    Station {
        code: "BLL",
        name: "Bardon Mill",
    },
    Station {
        code: "BLM",
        name: "Belmont",
    },
    Station {
        code: "BLN",
        name: "Blundellsands & Crosby",
    },
    Station {
        code: "BLO",
        name: "Blaydon",
    },
    Station {
        code: "BLP",
        name: "Belper",
    },
    Station {
        code: "BLT",
        name: "Blantyre",
    },
    Station {
        code: "BLV",
        name: "Belle Vue",
    },
    Station {
        code: "BLW",
        name: "Bulwell",
    },
    Station {
        code: "BLX",
        name: "Bloxwich",
    },
    Station {
        code: "BLY",
        name: "Bletchley",
    },
    Station {
        code: "BMA",
        name: "Ballymena (N Ireland)",
    },
    Station {
        code: "BMB",
        name: "Bamber Bridge",
    },
    Station {
        code: "BMC",
        name: "Bromley Cross",
    },
    Station {
        code: "BMD",
        name: "Brimsdown",
    },
    Station {
        code: "BME",
        name: "Broome",
    },
    Station {
        code: "BMF",
        name: "Broomfleet",
    },
    Station {
        code: "BMG",
        name: "Barming",
    },
    Station {
        code: "BMH",
        name: "Bournemouth",
    },
    Station {
        code: "BML",
        name: "Bramhall",
    },
    Station {
        code: "BMM",
        name: "Barlow Moor Road (Metrolink)",
    },
    Station {
        code: "BMN",
        name: "Bromley North",
    },
    Station {
        code: "BMO",
        name: "Birmingham Moor Street",
    },
    Station {
        code: "BMP",
        name: "Brampton (Cumbria)",
    },
    Station {
        code: "BMR",
        name: "Bromborough Rake",
    },
    Station {
        code: "BMS",
        name: "Bromley South",
    },
    Station {
        code: "BMT",
        name: "Bedminster",
    },
    Station {
        code: "BMV",
        name: "Bromsgrove",
    },
    Station {
        code: "BMY",
        name: "Bramley (Hants)",
    },
    Station {
        code: "BNA",
        name: "Burnage",
    },
    Station {
        code: "BNC",
        name: "Burnley Central",
    },
    Station {
        code: "BND",
        name: "Brandon",
    },
    Station {
        code: "BNE",
        name: "Bourne End",
    },
    Station {
        code: "BNF",
        name: "Briton Ferry",
    },
    Station {
        code: "BNG",
        name: "Bangor (Gwynedd)",
    },
    Station {
        code: "BNH",
        name: "Barnehurst",
    },
    Station {
        code: "BNI",
        name: "Barnes Bridge",
    },
    Station {
        code: "BNK",
        name: "Benchill (Metrolink)",
    },
    Station {
        code: "BNL",
        name: "Barnhill",
    },
    Station {
        code: "BNM",
        name: "Burnham (Bucks)",
    },
    Station {
        code: "BNP",
        name: "Barnstaple",
    },
    Station {
        code: "BNR",
        name: "Brockley Whins",
    },
    Station {
        code: "BNS",
        name: "Barnes",
    },
    Station {
        code: "BNT",
        name: "Brinnington",
    },
    Station {
        code: "BNU",
        name: "Banteer     (CIV)",
    },
    Station {
        code: "BNV",
        name: "Banavie",
    },
    Station {
        code: "BNW",
        name: "Bootle New Strand",
    },
    Station {
        code: "BNY",
        name: "Barnsley",
    },
    Station {
        code: "BOA",
        name: "Bradford-on-Avon",
    },
    Station {
        code: "BOB",
        name: "Besses o' th' Barn (Metrolink)",
    },
    Station {
        code: "BOC",
        name: "Bootle (Cumbria)",
    },
    Station {
        code: "BOD",
        name: "Bodmin Parkway",
    },
    Station {
        code: "BOE",
        name: "Botley",
    },
    Station {
        code: "BOG",
        name: "Bognor Regis",
    },
    Station {
        code: "BOH",
        name: "Bosham",
    },
    Station {
        code: "BOK",
        name: "Brooklands (Metrolink)",
    },
    Station {
        code: "BOM",
        name: "Bromborough",
    },
    Station {
        code: "BON",
        name: "Bolton",
    },
    Station {
        code: "BOP",
        name: "Bowes Park",
    },
    Station {
        code: "BOQ",
        name: "Boyle       (CIV)",
    },
    Station {
        code: "BOR",
        name: "Bodorgan",
    },
    Station {
        code: "BOT",
        name: "Bootle Oriel Rd",
    },
    Station {
        code: "BOW",
        name: "Bow Street",
    },
    Station {
        code: "BPB",
        name: "Blackpool Pleasure Beach",
    },
    Station {
        code: "BPK",
        name: "Brookmans Park",
    },
    Station {
        code: "BPL",
        name: "Barlaston Orchard Place (Bus)",
    },
    Station {
        code: "BPN",
        name: "Blackpool North",
    },
    Station {
        code: "BPO",
        name: "Bridport via First Bus X51/X53",
    },
    Station {
        code: "BPS",
        name: "Blackpool South",
    },
    Station {
        code: "BPT",
        name: "Bishopton (Renfrewshire)",
    },
    Station {
        code: "BPW",
        name: "Bristol Parkway",
    },
    Station {
        code: "BRA",
        name: "Brora",
    },
    Station {
        code: "BRC",
        name: "Breich",
    },
    Station {
        code: "BRD",
        name: "Broadway (Metrolink)",
    },
    Station {
        code: "BRE",
        name: "Brentwood",
    },
    Station {
        code: "BRF",
        name: "Brierfield",
    },
    Station {
        code: "BRG",
        name: "Borough Green & Wrotham",
    },
    Station {
        code: "BRH",
        name: "Borth",
    },
    Station {
        code: "BRI",
        name: "Bristol Temple Meads",
    },
    Station {
        code: "BRK",
        name: "Berwick (Sussex)",
    },
    Station {
        code: "BRL",
        name: "Barrhill",
    },
    Station {
        code: "BRM",
        name: "Barmouth",
    },
    Station {
        code: "BRN",
        name: "Bearsden",
    },
    Station {
        code: "BRO",
        name: "Bridge of Orchy",
    },
    Station {
        code: "BRP",
        name: "Brampton (Suffolk)",
    },
    Station {
        code: "BRR",
        name: "Barrhead",
    },
    Station {
        code: "BRS",
        name: "Berrylands",
    },
    Station {
        code: "BRT",
        name: "Barlaston",
    },
    Station {
        code: "BRU",
        name: "Bruton",
    },
    Station {
        code: "BRV",
        name: "Bournville",
    },
    Station {
        code: "BRW",
        name: "Brunswick",
    },
    Station {
        code: "BRX",
        name: "Brixton",
    },
    Station {
        code: "BRY",
        name: "Barry",
    },
    Station {
        code: "BRZ",
        name: "Burton Road (Metrolink)",
    },
    Station {
        code: "BSB",
        name: "Bleasby",
    },
    Station {
        code: "BSC",
        name: "Bescot Stadium",
    },
    Station {
        code: "BSD",
        name: "Bearsted",
    },
    Station {
        code: "BSE",
        name: "Bury St Edmunds",
    },
    Station {
        code: "BSG",
        name: "Ballinasloe (CIV)",
    },
    Station {
        code: "BSH",
        name: "Bushey",
    },
    Station {
        code: "BSI",
        name: "Balmossie",
    },
    Station {
        code: "BSJ",
        name: "Bedford St Johns",
    },
    Station {
        code: "BSK",
        name: "Basingstoke",
    },
    Station {
        code: "BSL",
        name: "Beasdale",
    },
    Station {
        code: "BSM",
        name: "Branksome",
    },
    Station {
        code: "BSN",
        name: "Boston",
    },
    Station {
        code: "BSO",
        name: "Basildon",
    },
    Station {
        code: "BSP",
        name: "Brondesbury Park",
    },
    Station {
        code: "BSR",
        name: "Broadstairs",
    },
    Station {
        code: "BSS",
        name: "Barassie",
    },
    Station {
        code: "BST",
        name: "Bishopstone Hillrise",
    },
    Station {
        code: "BSU",
        name: "Brunstane",
    },
    Station {
        code: "BSV",
        name: "Buckshaw Parkway",
    },
    Station {
        code: "BSW",
        name: "Birmingham Snow Hill",
    },
    Station {
        code: "BSY",
        name: "Brondesbury",
    },
    Station {
        code: "BTB",
        name: "Barnetby",
    },
    Station {
        code: "BTD",
        name: "Bolton-on-Dearne",
    },
    Station {
        code: "BTE",
        name: "Bitterne",
    },
    Station {
        code: "BTF",
        name: "Bottesford",
    },
    Station {
        code: "BTG",
        name: "Barnt Green",
    },
    Station {
        code: "BTH",
        name: "Bath Spa",
    },
    Station {
        code: "BTL",
        name: "Batley",
    },
    Station {
        code: "BTN",
        name: "Brighton",
    },
    Station {
        code: "BTO",
        name: "Betchworth",
    },
    Station {
        code: "BTP",
        name: "Braintree Freeport",
    },
    Station {
        code: "BTR",
        name: "Braintree",
    },
    Station {
        code: "BTS",
        name: "Burntisland",
    },
    Station {
        code: "BTT",
        name: "Battersby",
    },
    Station {
        code: "BTY",
        name: "Bentley (Hants)",
    },
    Station {
        code: "BUA",
        name: "Bude Bus",
    },
    Station {
        code: "BUB",
        name: "Burnley Barracks",
    },
    Station {
        code: "BUC",
        name: "Buckenham (Norfolk)",
    },
    Station {
        code: "BUD",
        name: "Burneside (Cumbria)",
    },
    Station {
        code: "BUE",
        name: "Bures",
    },
    Station {
        code: "BUG",
        name: "Burgess Hill",
    },
    Station {
        code: "BUH",
        name: "Brough",
    },
    Station {
        code: "BUI",
        name: "Burnside (Strathclyde)",
    },
    Station {
        code: "BUJ",
        name: "Burton Joyce",
    },
    Station {
        code: "BUK",
        name: "Bucknell",
    },
    Station {
        code: "BUL",
        name: "Butlers Lane",
    },
    Station {
        code: "BUO",
        name: "Bursledon",
    },
    Station {
        code: "BUR",
        name: "Bury (Metrolink)",
    },
    Station {
        code: "BUS",
        name: "Busby",
    },
    Station {
        code: "BUT",
        name: "Burton on Trent",
    },
    Station {
        code: "BUU",
        name: "Burnham-on-Crouch",
    },
    Station {
        code: "BUW",
        name: "Burley-in-Wharfedale",
    },
    Station {
        code: "BUX",
        name: "Buxton",
    },
    Station {
        code: "BUY",
        name: "Burley Park",
    },
    Station {
        code: "BVD",
        name: "Belvedere",
    },
    Station {
        code: "BWB",
        name: "Bow Brickhill",
    },
    Station {
        code: "BWD",
        name: "Birchwood",
    },
    Station {
        code: "BWG",
        name: "Bowling",
    },
    Station {
        code: "BWK",
        name: "Berwick-upon-Tweed",
    },
    Station {
        code: "BWN",
        name: "Bloxwich North",
    },
    Station {
        code: "BWO",
        name: "Bricket Wood",
    },
    Station {
        code: "BWS",
        name: "Barrow upon Soar",
    },
    Station {
        code: "BWT",
        name: "Bridgwater",
    },
    Station {
        code: "BXB",
        name: "Broxbourne",
    },
    Station {
        code: "BXD",
        name: "Buxted",
    },
    Station {
        code: "BXH",
        name: "Bexleyheath",
    },
    Station {
        code: "BXW",
        name: "Boxhill & Westhumble",
    },
    Station {
        code: "BXX",
        name: "Boxhill Burford Bridge",
    },
    Station {
        code: "BXY",
        name: "Bexley",
    },
    Station {
        code: "BYA",
        name: "Berney Arms",
    },
    Station {
        code: "BYB",
        name: "Blythe Bridge",
    },
    Station {
        code: "BYC",
        name: "Betws-y-Coed",
    },
    Station {
        code: "BYD",
        name: "Barry Docks",
    },
    Station {
        code: "BYE",
        name: "Bynea",
    },
    Station {
        code: "BYF",
        name: "Broughty Ferry",
    },
    Station {
        code: "BYI",
        name: "Barry Island",
    },
    Station {
        code: "BYK",
        name: "Bentley (South Yorks)",
    },
    Station {
        code: "BYL",
        name: "Barry Links",
    },
    Station {
        code: "BYM",
        name: "Burnley Manchester Rd",
    },
    Station {
        code: "BYN",
        name: "Bryn",
    },
    Station {
        code: "BYS",
        name: "Braystones (Cumbria)",
    },
    Station {
        code: "BZY",
        name: "Bray",
    },
    Station {
        code: "CAA",
        name: "Coventry Arena",
    },
    Station {
        code: "CAC",
        name: "Caldercruix",
    },
    Station {
        code: "CAD",
        name: "Cadoxton",
    },
    Station {
        code: "CAG",
        name: "Carrbridge",
    },
    Station {
        code: "CAH",
        name: "Cahir       (CIV)",
    },
    Station {
        code: "CAK",
        name: "Cark & Cartmel",
    },
    Station {
        code: "CAM",
        name: "Camberley",
    },
    Station {
        code: "CAN",
        name: "Carnoustie",
    },
    Station {
        code: "CAO",
        name: "Cannock",
    },
    Station {
        code: "CAR",
        name: "Carlisle",
    },
    Station {
        code: "CAS",
        name: "Castleton (Manchester)",
    },
    Station {
        code: "CAT",
        name: "Caterham",
    },
    Station {
        code: "CAU",
        name: "Causeland",
    },
    Station {
        code: "CAW",
        name: "Carlow      (CIV)",
    },
    Station {
        code: "CAY",
        name: "Carntyne",
    },
    Station {
        code: "CBB",
        name: "Carbis Bay",
    },
    Station {
        code: "CBC",
        name: "Coatbridge Central",
    },
    Station {
        code: "CBD",
        name: "Conon Bridge",
    },
    Station {
        code: "CBE",
        name: "Canterbury East",
    },
    Station {
        code: "CBG",
        name: "Cambridge",
    },
    Station {
        code: "CBH",
        name: "Cambridge Heath",
    },
    Station {
        code: "CBK",
        name: "Cranbrook (Devon)",
    },
    Station {
        code: "CBL",
        name: "Cambuslang",
    },
    Station {
        code: "CBN",
        name: "Camborne",
    },
    Station {
        code: "CBP",
        name: "Castle Bar Park",
    },
    Station {
        code: "CBR",
        name: "Cooksbridge",
    },
    Station {
        code: "CBS",
        name: "Coatbridge Sunnyside",
    },
    Station {
        code: "CBT",
        name: "Campbeltown",
    },
    Station {
        code: "CBW",
        name: "Canterbury West",
    },
    Station {
        code: "CBX",
        name: "CAMERON BRIDGE",
    },
    Station {
        code: "CBY",
        name: "Charlbury",
    },
    Station {
        code: "CBZ",
        name: "Corby Pboro Bus",
    },
    Station {
        code: "CCB",
        name: "Cardiff Central Bus Station",
    },
    Station {
        code: "CCC",
        name: "Criccieth",
    },
    Station {
        code: "CCH",
        name: "Chichester",
    },
    Station {
        code: "CCK",
        name: "CHINNOR RAIL",
    },
    Station {
        code: "CCN",
        name: "Castleconni",
    },
    Station {
        code: "CCT",
        name: "Cathcart",
    },
    Station {
        code: "CDB",
        name: "Cardiff Bay",
    },
    Station {
        code: "CDD",
        name: "Cardenden",
    },
    Station {
        code: "CDF",
        name: "Cardiff Central",
    },
    Station {
        code: "CDI",
        name: "Crediton",
    },
    Station {
        code: "CDN",
        name: "Coulsdon Town",
    },
    Station {
        code: "CDO",
        name: "Cardonald",
    },
    Station {
        code: "CDQ",
        name: "Cardiff Queen Street",
    },
    Station {
        code: "CDR",
        name: "Cardross",
    },
    Station {
        code: "CDS",
        name: "Coulsdon South",
    },
    Station {
        code: "CDT",
        name: "Caldicot",
    },
    Station {
        code: "CDU",
        name: "Cam & Dursley",
    },
    Station {
        code: "CDY",
        name: "Cartsdyke",
    },
    Station {
        code: "CEA",
        name: "Cleland",
    },
    Station {
        code: "CED",
        name: "Cheddington",
    },
    Station {
        code: "CEF",
        name: "Chapel-en-le-Frith",
    },
    Station {
        code: "CEH",
        name: "Coleshill Parkway",
    },
    Station {
        code: "CEI",
        name: "Coleraine (N Ireland)",
    },
    Station {
        code: "CEL",
        name: "Chelford (Cheshire)",
    },
    Station {
        code: "CEM",
        name: "Central Park (Metrolink)",
    },
    Station {
        code: "CES",
        name: "Cressing (Essex)",
    },
    Station {
        code: "CET",
        name: "Colchester Town",
    },
    Station {
        code: "CEY",
        name: "Cononley",
    },
    Station {
        code: "CFB",
        name: "Catford Bridge",
    },
    Station {
        code: "CFC",
        name: "Corfe Castle",
    },
    Station {
        code: "CFD",
        name: "Castleford",
    },
    Station {
        code: "CFF",
        name: "Croftfoot",
    },
    Station {
        code: "CFH",
        name: "Chafford Hundred",
    },
    Station {
        code: "CFL",
        name: "Crossflatts",
    },
    Station {
        code: "CFN",
        name: "Clifton Down",
    },
    Station {
        code: "CFO",
        name: "Chalfont & Latimer",
    },
    Station {
        code: "CFR",
        name: "Chandlers Ford",
    },
    Station {
        code: "CFT",
        name: "Crofton Park",
    },
    Station {
        code: "CGD",
        name: "Craigendoran",
    },
    Station {
        code: "CGM",
        name: "Cottingham",
    },
    Station {
        code: "CGN",
        name: "Cogan",
    },
    Station {
        code: "CGT",
        name: "Catterick Garrison Bus",
    },
    Station {
        code: "CGW",
        name: "Caergwrle",
    },
    Station {
        code: "CHB",
        name: "Clayton Hall (Metrolink)",
    },
    Station {
        code: "CHC",
        name: "Charing Cross (Glasgow)",
    },
    Station {
        code: "CHD",
        name: "Chesterfield",
    },
    Station {
        code: "CHE",
        name: "Cheam",
    },
    Station {
        code: "CHF",
        name: "Church Fenton",
    },
    Station {
        code: "CHG",
        name: "Charing (Kent)",
    },
    Station {
        code: "CHH",
        name: "Christs Hospital",
    },
    Station {
        code: "CHI",
        name: "Chingford",
    },
    Station {
        code: "CHJ",
        name: "Charleville (CIV)",
    },
    Station {
        code: "CHK",
        name: "Chiswick",
    },
    Station {
        code: "CHL",
        name: "Chilworth",
    },
    Station {
        code: "CHM",
        name: "Chelmsford",
    },
    Station {
        code: "CHN",
        name: "Cheshunt",
    },
    Station {
        code: "CHO",
        name: "Cholsey",
    },
    Station {
        code: "CHP",
        name: "Chipstead",
    },
    Station {
        code: "CHR",
        name: "Christchurch",
    },
    Station {
        code: "CHS",
        name: "CHURSTON",
    },
    Station {
        code: "CHT",
        name: "Chathill",
    },
    Station {
        code: "CHU",
        name: "Cheadle Hulme",
    },
    Station {
        code: "CHW",
        name: "Chalkwell",
    },
    Station {
        code: "CHX",
        name: "London Charing Cross",
    },
    Station {
        code: "CHY",
        name: "Chertsey",
    },
    Station {
        code: "CIL",
        name: "Chilham",
    },
    Station {
        code: "CIM",
        name: "Cilmeri",
    },
    Station {
        code: "CIR",
        name: "Caledonian Road & Barnsbury",
    },
    Station {
        code: "CIT",
        name: "Chislehurst",
    },
    Station {
        code: "CKA",
        name: "Carrick-on-Shannon (CIV)",
    },
    Station {
        code: "CKH",
        name: "Corkerhill",
    },
    Station {
        code: "CKL",
        name: "Corkickle",
    },
    Station {
        code: "CKN",
        name: "Crewkerne",
    },
    Station {
        code: "CKS",
        name: "Clarkston",
    },
    Station {
        code: "CKT",
        name: "Crookston",
    },
    Station {
        code: "CKU",
        name: "Carrick-on-Suir (CIV)",
    },
    Station {
        code: "CKY",
        name: "Crosskeys",
    },
    Station {
        code: "CLA",
        name: "Clandon",
    },
    Station {
        code: "CLB",
        name: "Castlebar   (CIV)",
    },
    Station {
        code: "CLC",
        name: "Castle Cary",
    },
    Station {
        code: "CLD",
        name: "Chelsfield",
    },
    Station {
        code: "CLE",
        name: "Cleethorpes",
    },
    Station {
        code: "CLG",
        name: "Claygate",
    },
    Station {
        code: "CLH",
        name: "Clitheroe",
    },
    Station {
        code: "CLI",
        name: "Clifton (Manchester)",
    },
    Station {
        code: "CLJ",
        name: "Clapham Junction",
    },
    Station {
        code: "CLK",
        name: "Clock House",
    },
    Station {
        code: "CLL",
        name: "Collington",
    },
    Station {
        code: "CLM",
        name: "Collingham",
    },
    Station {
        code: "CLN",
        name: "Chapeltown (Yorks)",
    },
    Station {
        code: "CLO",
        name: "COLL (ISLE OF)",
    },
    Station {
        code: "CLP",
        name: "Clapham High Street",
    },
    Station {
        code: "CLQ",
        name: "Clara       (CIV)",
    },
    Station {
        code: "CLR",
        name: "Clarbeston Road",
    },
    Station {
        code: "CLS",
        name: "Chester-le-Street",
    },
    Station {
        code: "CLT",
        name: "Clacton-on-Sea",
    },
    Station {
        code: "CLU",
        name: "Carluke",
    },
    Station {
        code: "CLV",
        name: "Claverdon",
    },
    Station {
        code: "CLW",
        name: "Chorleywood",
    },
    Station {
        code: "CLX",
        name: "Clonmel     (CIV)",
    },
    Station {
        code: "CLY",
        name: "Chinley",
    },
    Station {
        code: "CLZ",
        name: "Cloughjordan (CIV)",
    },
    Station {
        code: "CMB",
        name: "Cambridge North",
    },
    Station {
        code: "CMD",
        name: "Camden Road",
    },
    Station {
        code: "CME",
        name: "Combe (Oxon)",
    },
    Station {
        code: "CMF",
        name: "Cromford",
    },
    Station {
        code: "CMH",
        name: "Cwmbach",
    },
    Station {
        code: "CMI",
        name: "Claremorris (CIV)",
    },
    Station {
        code: "CMK",
        name: "Crossacres (Metrolink)",
    },
    Station {
        code: "CML",
        name: "Carmyle",
    },
    Station {
        code: "CMN",
        name: "Carmarthen",
    },
    Station {
        code: "CMO",
        name: "Camelon",
    },
    Station {
        code: "CMR",
        name: "Cromer",
    },
    Station {
        code: "CMY",
        name: "Crossmyloof",
    },
    Station {
        code: "CNA",
        name: "CANNA (ISLE OF)",
    },
    Station {
        code: "CNE",
        name: "Colne",
    },
    Station {
        code: "CNF",
        name: "Carnforth",
    },
    Station {
        code: "CNG",
        name: "Congleton",
    },
    Station {
        code: "CNK",
        name: "Chorlton (Metrolink)",
    },
    Station {
        code: "CNL",
        name: "Canley",
    },
    Station {
        code: "CNM",
        name: "Cheltenham Spa",
    },
    Station {
        code: "CNN",
        name: "Canonbury",
    },
    Station {
        code: "CNO",
        name: "Chetnole",
    },
    Station {
        code: "CNP",
        name: "Conway Park",
    },
    Station {
        code: "CNR",
        name: "Crianlarich",
    },
    Station {
        code: "CNS",
        name: "Conisbrough",
    },
    Station {
        code: "CNW",
        name: "Conwy",
    },
    Station {
        code: "CNY",
        name: "Cantley",
    },
    Station {
        code: "COA",
        name: "Coatdyke",
    },
    Station {
        code: "COB",
        name: "Cooden Beach",
    },
    Station {
        code: "COC",
        name: "Cowden Crossroads",
    },
    Station {
        code: "COE",
        name: "Coombe (Cornwall)",
    },
    Station {
        code: "COH",
        name: "Crowborough",
    },
    Station {
        code: "COI",
        name: "Crosshill",
    },
    Station {
        code: "COK",
        name: "Cork City   (CIV)",
    },
    Station {
        code: "COL",
        name: "Colchester",
    },
    Station {
        code: "COM",
        name: "Commondale",
    },
    Station {
        code: "CON",
        name: "Connel Ferry",
    },
    Station {
        code: "COO",
        name: "Cookham",
    },
    Station {
        code: "COP",
        name: "Copplestone",
    },
    Station {
        code: "COQ",
        name: "Cobh        (CIV)",
    },
    Station {
        code: "COR",
        name: "Corby",
    },
    Station {
        code: "COS",
        name: "Cosford",
    },
    Station {
        code: "COT",
        name: "Cottingley",
    },
    Station {
        code: "COU",
        name: "Collooney   (CIV)",
    },
    Station {
        code: "COV",
        name: "Coventry",
    },
    Station {
        code: "COW",
        name: "Cowdenbeath",
    },
    Station {
        code: "COY",
        name: "Coryton",
    },
    Station {
        code: "COZ",
        name: "Cornbrook (Metrolink)",
    },
    Station {
        code: "CPA",
        name: "Corpach",
    },
    Station {
        code: "CPG",
        name: "CHIPNGNORTONBUS",
    },
    Station {
        code: "CPH",
        name: "Caerphilly",
    },
    Station {
        code: "CPK",
        name: "Carpenders Park",
    },
    Station {
        code: "CPM",
        name: "Chippenham",
    },
    Station {
        code: "CPN",
        name: "Chapelton (Devon)",
    },
    Station {
        code: "CPT",
        name: "Clapton",
    },
    Station {
        code: "CPU",
        name: "Capenhurst",
    },
    Station {
        code: "CPW",
        name: "Chepstow",
    },
    Station {
        code: "CPY",
        name: "Clapham (Yorks)",
    },
    Station {
        code: "CRA",
        name: "Cradley Heath",
    },
    Station {
        code: "CRB",
        name: "Corbridge",
    },
    Station {
        code: "CRC",
        name: "Cheltenham Races Bus",
    },
    Station {
        code: "CRD",
        name: "Chester Road",
    },
    Station {
        code: "CRE",
        name: "Crewe",
    },
    Station {
        code: "CRF",
        name: "Carfin",
    },
    Station {
        code: "CRG",
        name: "Cross Gates",
    },
    Station {
        code: "CRH",
        name: "Crouch Hill",
    },
    Station {
        code: "CRI",
        name: "Cricklewood",
    },
    Station {
        code: "CRJ",
        name: "Crumpsall (Metrolink)",
    },
    Station {
        code: "CRK",
        name: "Chirk",
    },
    Station {
        code: "CRL",
        name: "Chorley",
    },
    Station {
        code: "CRM",
        name: "Cramlington",
    },
    Station {
        code: "CRN",
        name: "Crowthorne",
    },
    Station {
        code: "CRO",
        name: "Croy",
    },
    Station {
        code: "CRP",
        name: "Cairnryan",
    },
    Station {
        code: "CRQ",
        name: "Cemetry Road (Metrolink)",
    },
    Station {
        code: "CRR",
        name: "Corrour",
    },
    Station {
        code: "CRS",
        name: "Carstairs",
    },
    Station {
        code: "CRT",
        name: "Chartham",
    },
    Station {
        code: "CRU",
        name: "Craignure",
    },
    Station {
        code: "CRV",
        name: "Craven Arms",
    },
    Station {
        code: "CRW",
        name: "Crawley",
    },
    Station {
        code: "CRY",
        name: "Crayford",
    },
    Station {
        code: "CRZ",
        name: "Cricklade Bus",
    },
    Station {
        code: "CSA",
        name: "Cosham",
    },
    Station {
        code: "CSB",
        name: "Carshalton Beech",
    },
    Station {
        code: "CSD",
        name: "Cobham & Stoke D'Abernon",
    },
    Station {
        code: "CSE",
        name: "Castlerea   (CIV)",
    },
    Station {
        code: "CSG",
        name: "Cressington",
    },
    Station {
        code: "CSH",
        name: "Carshalton",
    },
    Station {
        code: "CSK",
        name: "Calstock",
    },
    Station {
        code: "CSL",
        name: "Codsall",
    },
    Station {
        code: "CSM",
        name: "Castleton Moor",
    },
    Station {
        code: "CSN",
        name: "Chessington North",
    },
    Station {
        code: "CSO",
        name: "Croston",
    },
    Station {
        code: "CSR",
        name: "Chassen Road",
    },
    Station {
        code: "CSS",
        name: "Chessington South",
    },
    Station {
        code: "CST",
        name: "London Cannon Street",
    },
    Station {
        code: "CSW",
        name: "Chestfield & Swalecliffe",
    },
    Station {
        code: "CSY",
        name: "Coseley",
    },
    Station {
        code: "CTB",
        name: "Castlebay",
    },
    Station {
        code: "CTE",
        name: "Chatelherault",
    },
    Station {
        code: "CTF",
        name: "Catford",
    },
    Station {
        code: "CTH",
        name: "Chadwell Heath",
    },
    Station {
        code: "CTK",
        name: "City Thameslink",
    },
    Station {
        code: "CTL",
        name: "Cattal",
    },
    Station {
        code: "CTM",
        name: "Chatham",
    },
    Station {
        code: "CTN",
        name: "Charlton",
    },
    Station {
        code: "CTO",
        name: "Carlton",
    },
    Station {
        code: "CTR",
        name: "Chester",
    },
    Station {
        code: "CTT",
        name: "Church Stretton",
    },
    Station {
        code: "CTW",
        name: "Church & Oswaldtwistle",
    },
    Station {
        code: "CUA",
        name: "Culrain",
    },
    Station {
        code: "CUB",
        name: "Cumbernauld",
    },
    Station {
        code: "CUD",
        name: "Cuddington",
    },
    Station {
        code: "CUF",
        name: "Cuffley",
    },
    Station {
        code: "CUH",
        name: "Curriehill",
    },
    Station {
        code: "CUL",
        name: "Cumbrae",
    },
    Station {
        code: "CUM",
        name: "Culham",
    },
    Station {
        code: "CUP",
        name: "Cupar",
    },
    Station {
        code: "CUS",
        name: "Custom House (Elizabeth line)",
    },
    Station {
        code: "CUW",
        name: "Clunderwen",
    },
    Station {
        code: "CUX",
        name: "Cuxton",
    },
    Station {
        code: "CVG",
        name: "Charlbury Village",
    },
    Station {
        code: "CWB",
        name: "Colwyn Bay",
    },
    Station {
        code: "CWC",
        name: "Chappel & Wakes Colne",
    },
    Station {
        code: "CWD",
        name: "Creswell",
    },
    Station {
        code: "CWE",
        name: "Crowle",
    },
    Station {
        code: "CWH",
        name: "Crews Hill",
    },
    Station {
        code: "CWL",
        name: "Colwall",
    },
    Station {
        code: "CWM",
        name: "Cwmbran",
    },
    Station {
        code: "CWN",
        name: "Cowden (Kent)",
    },
    Station {
        code: "CWS",
        name: "Caersws",
    },
    Station {
        code: "CWU",
        name: "Crowhurst",
    },
    Station {
        code: "CWX",
        name: "Canary Wharf (Elizabeth line)",
    },
    Station {
        code: "CYB",
        name: "Cefn Y Bedd",
    },
    Station {
        code: "CYK",
        name: "Clydebank",
    },
    Station {
        code: "CYN",
        name: "Cynghordy",
    },
    Station {
        code: "CYP",
        name: "Crystal Palace",
    },
    Station {
        code: "CYS",
        name: "Cathays",
    },
    Station {
        code: "CYT",
        name: "Cherry Tree",
    },
    Station {
        code: "DAG",
        name: "Dalgety Bay",
    },
    Station {
        code: "DAK",
        name: "Dalmarnock",
    },
    Station {
        code: "DAL",
        name: "Dalmally",
    },
    Station {
        code: "DAM",
        name: "Dalmeny",
    },
    Station {
        code: "DAN",
        name: "Darnall",
    },
    Station {
        code: "DAR",
        name: "Darlington",
    },
    Station {
        code: "DAT",
        name: "Datchet",
    },
    Station {
        code: "DBC",
        name: "Dumbarton Central",
    },
    Station {
        code: "DBD",
        name: "Denby Dale",
    },
    Station {
        code: "DBE",
        name: "Dumbarton East",
    },
    Station {
        code: "DBG",
        name: "Mottisfont & Dunbridge",
    },
    Station {
        code: "DBL",
        name: "Dunblane",
    },
    Station {
        code: "DBP",
        name: "Dublin Pearse (CIV)",
    },
    Station {
        code: "DBR",
        name: "Derby Road (Ipswich)",
    },
    Station {
        code: "DBY",
        name: "Derby",
    },
    Station {
        code: "DCG",
        name: "Duncraig",
    },
    Station {
        code: "DCH",
        name: "Dorchester South",
    },
    Station {
        code: "DCL",
        name: "Dublin Connolly (CIV)",
    },
    Station {
        code: "DCT",
        name: "Danes Court",
    },
    Station {
        code: "DCW",
        name: "Dorchester West",
    },
    Station {
        code: "DDB",
        name: "DUN LAOIRE  (CIV",
    },
    Station {
        code: "DDG",
        name: "Dorridge",
    },
    Station {
        code: "DDK",
        name: "Dagenham Dock",
    },
    Station {
        code: "DDP",
        name: "Dudley Port",
    },
    Station {
        code: "DEA",
        name: "Deal",
    },
    Station {
        code: "DEB",
        name: "Dereham Market Place",
    },
    Station {
        code: "DEE",
        name: "Dundee",
    },
    Station {
        code: "DEN",
        name: "Dean (Wilts)",
    },
    Station {
        code: "DEP",
        name: "Deptford",
    },
    Station {
        code: "DEW",
        name: "Dewsbury",
    },
    Station {
        code: "DFD",
        name: "Dartford",
    },
    Station {
        code: "DFE",
        name: "Dunfermline City",
    },
    Station {
        code: "DFI",
        name: "Duffield",
    },
    Station {
        code: "DFL",
        name: "Dunfermline Queen Margaret",
    },
    Station {
        code: "DFP",
        name: "Dublin Ferryport (Irish Ferries)",
    },
    Station {
        code: "DFR",
        name: "Drumfrochar",
    },
    Station {
        code: "DGC",
        name: "Denham Golf Club",
    },
    Station {
        code: "DGL",
        name: "Dingle Road",
    },
    Station {
        code: "DGS",
        name: "Douglas (Isle of Man)",
    },
    Station {
        code: "DGT",
        name: "Deansgate G-Mex",
    },
    Station {
        code: "DGY",
        name: "Deganwy",
    },
    Station {
        code: "DHM",
        name: "Durham",
    },
    Station {
        code: "DHN",
        name: "Deighton",
    },
    Station {
        code: "DHT",
        name: "Dublin Heuston (CIV)",
    },
    Station {
        code: "DID",
        name: "Didcot Parkway",
    },
    Station {
        code: "DIG",
        name: "Digby & Sowton",
    },
    Station {
        code: "DIN",
        name: "Dingwall",
    },
    Station {
        code: "DIS",
        name: "Diss",
    },
    Station {
        code: "DKD",
        name: "Dunkeld & Birnam",
    },
    Station {
        code: "DKG",
        name: "Dorking",
    },
    Station {
        code: "DKR",
        name: "Derker (Metrolink)",
    },
    Station {
        code: "DKT",
        name: "Dorking West",
    },
    Station {
        code: "DLE",
        name: "Duloe (Causeland)",
    },
    Station {
        code: "DLG",
        name: "Dolgarrog",
    },
    Station {
        code: "DLH",
        name: "Doleham",
    },
    Station {
        code: "DLJ",
        name: "Dalston Junction",
    },
    Station {
        code: "DLK",
        name: "Dalston Kingsland",
    },
    Station {
        code: "DLM",
        name: "Delamere",
    },
    Station {
        code: "DLR",
        name: "Dalreoch",
    },
    Station {
        code: "DLS",
        name: "Dalston (Cumbria)",
    },
    Station {
        code: "DLT",
        name: "Dalton (Cumbria)",
    },
    Station {
        code: "DLW",
        name: "Dalwhinnie",
    },
    Station {
        code: "DLY",
        name: "Dalry",
    },
    Station {
        code: "DMC",
        name: "Drumchapel",
    },
    Station {
        code: "DMD",
        name: "Dromod      (CIV)",
    },
    Station {
        code: "DMF",
        name: "Dumfries",
    },
    Station {
        code: "DMG",
        name: "Dinas (Rhondda)",
    },
    Station {
        code: "DMH",
        name: "Dilton Marsh",
    },
    Station {
        code: "DMK",
        name: "Denmark Hill",
    },
    Station {
        code: "DML",
        name: "Droylesden (Metrolink)",
    },
    Station {
        code: "DMP",
        name: "Dumpton Park",
    },
    Station {
        code: "DMR",
        name: "Dalmuir",
    },
    Station {
        code: "DMS",
        name: "Dormans",
    },
    Station {
        code: "DMY",
        name: "Drumry",
    },
    Station {
        code: "DND",
        name: "Dinsdale",
    },
    Station {
        code: "DNG",
        name: "Dunton Green",
    },
    Station {
        code: "DNL",
        name: "Dunlop",
    },
    Station {
        code: "DNM",
        name: "Denham",
    },
    Station {
        code: "DNO",
        name: "Dunrobin Castle",
    },
    Station {
        code: "DNS",
        name: "Dinas Powys",
    },
    Station {
        code: "DNT",
        name: "Dent",
    },
    Station {
        code: "DNY",
        name: "Danby",
    },
    Station {
        code: "DOC",
        name: "Dockyard Devonport",
    },
    Station {
        code: "DOD",
        name: "Dodworth",
    },
    Station {
        code: "DOL",
        name: "Dolau",
    },
    Station {
        code: "DON",
        name: "Doncaster",
    },
    Station {
        code: "DOR",
        name: "Dore & Totley",
    },
    Station {
        code: "DOT",
        name: "Dunston",
    },
    Station {
        code: "DOW",
        name: "Downham Market",
    },
    Station {
        code: "DPD",
        name: "Dorking (Deepdene)",
    },
    Station {
        code: "DPS",
        name: "Dublin Port Stena",
    },
    Station {
        code: "DPT",
        name: "Devonport (Devon)",
    },
    Station {
        code: "DRA",
        name: "Drogheda    (CIV)",
    },
    Station {
        code: "DRD",
        name: "Dane Road (Metrolink)",
    },
    Station {
        code: "DRF",
        name: "Driffield",
    },
    Station {
        code: "DRG",
        name: "Drayton Green",
    },
    Station {
        code: "DRI",
        name: "Drigg",
    },
    Station {
        code: "DRM",
        name: "Drem",
    },
    Station {
        code: "DRN",
        name: "Duirinish",
    },
    Station {
        code: "DRO",
        name: "Dronfield",
    },
    Station {
        code: "DRT",
        name: "Darton",
    },
    Station {
        code: "DRU",
        name: "Drumgelloch",
    },
    Station {
        code: "DSL",
        name: "Disley",
    },
    Station {
        code: "DSM",
        name: "Darsham",
    },
    Station {
        code: "DST",
        name: "Duke Street",
    },
    Station {
        code: "DSY",
        name: "Daisy Hill",
    },
    Station {
        code: "DTG",
        name: "Dinting",
    },
    Station {
        code: "DTN",
        name: "Denton",
    },
    Station {
        code: "DTW",
        name: "Droitwich Spa",
    },
    Station {
        code: "DUD",
        name: "Duddeston",
    },
    Station {
        code: "DUK",
        name: "Dundalk (CIV)",
    },
    Station {
        code: "DUL",
        name: "Dullingham",
    },
    Station {
        code: "DUM",
        name: "Dumbreck",
    },
    Station {
        code: "DUN",
        name: "Dunbar",
    },
    Station {
        code: "DUO",
        name: "Dunoon",
    },
    Station {
        code: "DUR",
        name: "Durrington",
    },
    Station {
        code: "DUU",
        name: "Duns",
    },
    Station {
        code: "DVC",
        name: "Dovercourt",
    },
    Station {
        code: "DVH",
        name: "Dove Holes",
    },
    Station {
        code: "DVM",
        name: "Didsbury Village (Metrolink)",
    },
    Station {
        code: "DVN",
        name: "Davenport",
    },
    Station {
        code: "DVP",
        name: "Dover Priory",
    },
    Station {
        code: "DVR",
        name: "Daventry Bus",
    },
    Station {
        code: "DVY",
        name: "Dovey Junction",
    },
    Station {
        code: "DWD",
        name: "Dolwyddelan",
    },
    Station {
        code: "DWL",
        name: "Dawlish",
    },
    Station {
        code: "DWN",
        name: "Darwen",
    },
    Station {
        code: "DWW",
        name: "Dawlish Warren",
    },
    Station {
        code: "DYC",
        name: "Dyce",
    },
    Station {
        code: "DYF",
        name: "Dyffryn Ardudwy",
    },
    Station {
        code: "DYP",
        name: "Drayton Park",
    },
    Station {
        code: "DZY",
        name: "Danzey",
    },
    Station {
        code: "EAD",
        name: "Earlsfield",
    },
    Station {
        code: "EAG",
        name: "Eaglescliffe",
    },
    Station {
        code: "EAL",
        name: "Ealing Broadway",
    },
    Station {
        code: "EAR",
        name: "Earley",
    },
    Station {
        code: "EAS",
        name: "Earlston",
    },
    Station {
        code: "EBA",
        name: "Euxton Balshaw Lane",
    },
    Station {
        code: "EBB",
        name: "Ebbw Vale Town",
    },
    Station {
        code: "EBD",
        name: "Ebbsfleet International",
    },
    Station {
        code: "EBF",
        name: "Ebbsfleet International (CIV)",
    },
    Station {
        code: "EBK",
        name: "Eastbrook",
    },
    Station {
        code: "EBL",
        name: "East Boldon",
    },
    Station {
        code: "EBN",
        name: "Eastbourne",
    },
    Station {
        code: "EBR",
        name: "Edenbridge",
    },
    Station {
        code: "EBT",
        name: "Edenbridge Town",
    },
    Station {
        code: "EBV",
        name: "Ebbw Vale Parkway",
    },
    Station {
        code: "ECC",
        name: "Eccles (Manchester)",
    },
    Station {
        code: "ECL",
        name: "Eccleston Park",
    },
    Station {
        code: "ECM",
        name: "Eccles (Metrolink)",
    },
    Station {
        code: "ECP",
        name: "Energlyn & Churchill Park",
    },
    Station {
        code: "ECR",
        name: "East Croydon",
    },
    Station {
        code: "ECS",
        name: "Eccles Road",
    },
    Station {
        code: "ECW",
        name: "Cowes East (Red Funnel Ship)",
    },
    Station {
        code: "EDA",
        name: "Edinburgh Airport Bus/Tram",
    },
    Station {
        code: "EDB",
        name: "Edinburgh",
    },
    Station {
        code: "EDG",
        name: "Edge Hill",
    },
    Station {
        code: "EDL",
        name: "Edale",
    },
    Station {
        code: "EDM",
        name: "East Didsbury (Metrolink)",
    },
    Station {
        code: "EDN",
        name: "Eden Park",
    },
    Station {
        code: "EDP",
        name: "Edinburgh Park",
    },
    Station {
        code: "EDR",
        name: "Edmonton Green",
    },
    Station {
        code: "EDW",
        name: "East Dulwich",
    },
    Station {
        code: "EDY",
        name: "East Didsbury",
    },
    Station {
        code: "EFD",
        name: "Enfield     (CIV)",
    },
    Station {
        code: "EFF",
        name: "Effingham Junction",
    },
    Station {
        code: "EFL",
        name: "East Farleigh",
    },
    Station {
        code: "EGF",
        name: "East Garforth",
    },
    Station {
        code: "EGG",
        name: "Eggesford",
    },
    Station {
        code: "EGH",
        name: "Egham",
    },
    Station {
        code: "EGN",
        name: "Eastrington",
    },
    Station {
        code: "EGR",
        name: "East Grinstead",
    },
    Station {
        code: "EGT",
        name: "Egton",
    },
    Station {
        code: "EGY",
        name: "Edinburgh Gateway",
    },
    Station {
        code: "EHC",
        name: "Etihad Camp (Metrolink)",
    },
    Station {
        code: "EIG",
        name: "EIGG (ISLE OF)",
    },
    Station {
        code: "EKB",
        name: "Eskbank",
    },
    Station {
        code: "EKL",
        name: "East Kilbride",
    },
    Station {
        code: "EKR",
        name: "EYTHORNE EKR",
    },
    Station {
        code: "ELD",
        name: "Earlswood (Surrey)",
    },
    Station {
        code: "ELE",
        name: "Elmers End",
    },
    Station {
        code: "ELG",
        name: "Elgin",
    },
    Station {
        code: "ELM",
        name: "Edge Lane (Metrolink)",
    },
    Station {
        code: "ELN",
        name: "ELLAND",
    },
    Station {
        code: "ELO",
        name: "Elton & Orston",
    },
    Station {
        code: "ELP",
        name: "Ellesmere Port",
    },
    Station {
        code: "ELR",
        name: "Elsecar",
    },
    Station {
        code: "ELS",
        name: "Elstree & Borehamwood",
    },
    Station {
        code: "ELT",
        name: "East Linton",
    },
    Station {
        code: "ELW",
        name: "Eltham",
    },
    Station {
        code: "ELY",
        name: "Ely",
    },
    Station {
        code: "EMA",
        name: "EAST M AIR/DBY",
    },
    Station {
        code: "EMC",
        name: "EAST M AIR/EMD",
    },
    Station {
        code: "EMD",
        name: "East Midlands Parkway",
    },
    Station {
        code: "EML",
        name: "East Malling",
    },
    Station {
        code: "EMP",
        name: "Emerson Park",
    },
    Station {
        code: "EMS",
        name: "Emsworth",
    },
    Station {
        code: "ENC",
        name: "Enfield Chase",
    },
    Station {
        code: "ENF",
        name: "Enfield Town",
    },
    Station {
        code: "ENL",
        name: "Enfield Lock",
    },
    Station {
        code: "ENN",
        name: "Ennis       (CIV)",
    },
    Station {
        code: "ENS",
        name: "Enniscorthy (CIV)",
    },
    Station {
        code: "ENT",
        name: "Entwistle",
    },
    Station {
        code: "EPD",
        name: "Epsom Downs",
    },
    Station {
        code: "EPH",
        name: "Elephant & Castle",
    },
    Station {
        code: "EPS",
        name: "Epsom",
    },
    Station {
        code: "ERA",
        name: "Eastham Rake",
    },
    Station {
        code: "ERB",
        name: "Eridge A26 Bus S",
    },
    Station {
        code: "ERD",
        name: "Erdington",
    },
    Station {
        code: "ERH",
        name: "Erith",
    },
    Station {
        code: "ERI",
        name: "Eridge",
    },
    Station {
        code: "ERL",
        name: "Earlestown",
    },
    Station {
        code: "ESD",
        name: "Elmstead Woods",
    },
    Station {
        code: "ESH",
        name: "Esher",
    },
    Station {
        code: "ESL",
        name: "Eastleigh",
    },
    Station {
        code: "ESM",
        name: "Elsenham Essex",
    },
    Station {
        code: "EST",
        name: "Easterhouse",
    },
    Station {
        code: "ESW",
        name: "Elmswell",
    },
    Station {
        code: "ETC",
        name: "Etchingham",
    },
    Station {
        code: "ETL",
        name: "East Tilbury",
    },
    Station {
        code: "EUS",
        name: "London Euston",
    },
    Station {
        code: "EVE",
        name: "Evesham",
    },
    Station {
        code: "EWD",
        name: "Earlswood (West Midlands)",
    },
    Station {
        code: "EWE",
        name: "Ewell East",
    },
    Station {
        code: "EWR",
        name: "East Worthing",
    },
    Station {
        code: "EWW",
        name: "Ewell West",
    },
    Station {
        code: "EXC",
        name: "Exeter Central",
    },
    Station {
        code: "EXD",
        name: "Exeter St Davids",
    },
    Station {
        code: "EXG",
        name: "Exhibition Centre (Glasgow)",
    },
    Station {
        code: "EXM",
        name: "Exmouth",
    },
    Station {
        code: "EXN",
        name: "Exton",
    },
    Station {
        code: "EXQ",
        name: "Exchange Quay (Metrolink)",
    },
    Station {
        code: "EXR",
        name: "Essex Road",
    },
    Station {
        code: "EXT",
        name: "Exeter St Thomas",
    },
    Station {
        code: "EYN",
        name: "Eynsford",
    },
    Station {
        code: "FAL",
        name: "Falmouth Docks",
    },
    Station {
        code: "FAR",
        name: "Farranfore  (CIV)",
    },
    Station {
        code: "FAV",
        name: "Faversham",
    },
    Station {
        code: "FAX",
        name: "Faringdon Bus",
    },
    Station {
        code: "FAZ",
        name: "Fazakerley",
    },
    Station {
        code: "FBY",
        name: "Formby",
    },
    Station {
        code: "FCN",
        name: "Falconwood",
    },
    Station {
        code: "FEA",
        name: "Featherstone",
    },
    Station {
        code: "FEG",
        name: "Fellgate Metro",
    },
    Station {
        code: "FEL",
        name: "Feltham",
    },
    Station {
        code: "FEN",
        name: "Fenny Stratford",
    },
    Station {
        code: "FER",
        name: "Fernhill",
    },
    Station {
        code: "FFA",
        name: "Ffairfach",
    },
    Station {
        code: "FFD",
        name: "Freshford",
    },
    Station {
        code: "FGH",
        name: "Fishguard Harbour",
    },
    Station {
        code: "FGT",
        name: "Faygate",
    },
    Station {
        code: "FGW",
        name: "Fishguard & Goodwick",
    },
    Station {
        code: "FHM",
        name: "Freehold (Metrolink)",
    },
    Station {
        code: "FIL",
        name: "Filey",
    },
    Station {
        code: "FIN",
        name: "Finstock",
    },
    Station {
        code: "FIT",
        name: "Filton Abbeywood",
    },
    Station {
        code: "FKC",
        name: "Folkestone Central",
    },
    Station {
        code: "FKG",
        name: "Falkirk Grahamston",
    },
    Station {
        code: "FKK",
        name: "Falkirk High",
    },
    Station {
        code: "FKW",
        name: "Folkestone West",
    },
    Station {
        code: "FLD",
        name: "Fauldhouse",
    },
    Station {
        code: "FLE",
        name: "Fleet",
    },
    Station {
        code: "FLF",
        name: "Flowery Field",
    },
    Station {
        code: "FLI",
        name: "Flixton",
    },
    Station {
        code: "FLM",
        name: "Flimby",
    },
    Station {
        code: "FLN",
        name: "Flint",
    },
    Station {
        code: "FLS",
        name: "Failsworth (Metrolink)",
    },
    Station {
        code: "FLT",
        name: "Flitwick",
    },
    Station {
        code: "FLW",
        name: "Fulwell",
    },
    Station {
        code: "FLX",
        name: "Felixstowe",
    },
    Station {
        code: "FLZ",
        name: "Flamingoland Bus",
    },
    Station {
        code: "FML",
        name: "Frimley",
    },
    Station {
        code: "FMR",
        name: "Falmer",
    },
    Station {
        code: "FMT",
        name: "Falmouth Town",
    },
    Station {
        code: "FNB",
        name: "Farnborough Main",
    },
    Station {
        code: "FNC",
        name: "Farncombe",
    },
    Station {
        code: "FNH",
        name: "Farnham",
    },
    Station {
        code: "FNN",
        name: "Farnborough North",
    },
    Station {
        code: "FNR",
        name: "Farningham Road",
    },
    Station {
        code: "FNT",
        name: "Feniton",
    },
    Station {
        code: "FNV",
        name: "Furness Vale",
    },
    Station {
        code: "FNW",
        name: "Farnworth",
    },
    Station {
        code: "FNY",
        name: "Finchley Road & Frognal",
    },
    Station {
        code: "FOC",
        name: "Falls of Cruachan",
    },
    Station {
        code: "FOD",
        name: "Ford",
    },
    Station {
        code: "FOG",
        name: "Forest Gate",
    },
    Station {
        code: "FOH",
        name: "Forest Hill",
    },
    Station {
        code: "FOK",
        name: "Four Oaks",
    },
    Station {
        code: "FOR",
        name: "Forres",
    },
    Station {
        code: "FOT",
        name: "Fota        (CIV)",
    },
    Station {
        code: "FOX",
        name: "Foxfield",
    },
    Station {
        code: "FPK",
        name: "Finsbury Park",
    },
    Station {
        code: "FRB",
        name: "Fairbourne",
    },
    Station {
        code: "FRD",
        name: "Frodsham",
    },
    Station {
        code: "FRE",
        name: "Freshfield",
    },
    Station {
        code: "FRF",
        name: "Fairfield",
    },
    Station {
        code: "FRI",
        name: "Frinton-on-Sea",
    },
    Station {
        code: "FRL",
        name: "Fairlie",
    },
    Station {
        code: "FRM",
        name: "Fareham",
    },
    Station {
        code: "FRN",
        name: "Fearn",
    },
    Station {
        code: "FRO",
        name: "Frome",
    },
    Station {
        code: "FRS",
        name: "Forsinard",
    },
    Station {
        code: "FRT",
        name: "Frant",
    },
    Station {
        code: "FRW",
        name: "Fairwater",
    },
    Station {
        code: "FRY",
        name: "Ferriby",
    },
    Station {
        code: "FSB",
        name: "Fishbourne (Sussex)",
    },
    Station {
        code: "FSG",
        name: "Fishersgate",
    },
    Station {
        code: "FSK",
        name: "Fiskerton",
    },
    Station {
        code: "FST",
        name: "London Fenchurch Street",
    },
    Station {
        code: "FTM",
        name: "Fort Matilda",
    },
    Station {
        code: "FTN",
        name: "Fratton",
    },
    Station {
        code: "FTW",
        name: "Fort William",
    },
    Station {
        code: "FWN",
        name: "Firswood (Metrolink)",
    },
    Station {
        code: "FWY",
        name: "Five Ways",
    },
    Station {
        code: "FXF",
        name: "Foxford",
    },
    Station {
        code: "FXN",
        name: "Foxton",
    },
    Station {
        code: "FYS",
        name: "Ferryside",
    },
    Station {
        code: "FZH",
        name: "Frizinghall",
    },
    Station {
        code: "FZP",
        name: "Furze Platt",
    },
    Station {
        code: "FZW",
        name: "Fitzwilliam",
    },
    Station {
        code: "GAL",
        name: "Galashiels",
    },
    Station {
        code: "GAR",
        name: "Garrowhill",
    },
    Station {
        code: "GBD",
        name: "Gilberdyke",
    },
    Station {
        code: "GBG",
        name: "Gorebridge",
    },
    Station {
        code: "GBK",
        name: "Greenbank",
    },
    Station {
        code: "GBL",
        name: "Gainsborough Lea Road",
    },
    Station {
        code: "GBS",
        name: "Goring by Sea",
    },
    Station {
        code: "GCH",
        name: "Garelochhead",
    },
    Station {
        code: "GCR",
        name: "Gloucester",
    },
    Station {
        code: "GCT",
        name: "Great Coates",
    },
    Station {
        code: "GCW",
        name: "Glan Conwy",
    },
    Station {
        code: "GDH",
        name: "Gordon Hill",
    },
    Station {
        code: "GDL",
        name: "Godley",
    },
    Station {
        code: "GDN",
        name: "Godstone",
    },
    Station {
        code: "GDP",
        name: "Gidea Park",
    },
    Station {
        code: "GEA",
        name: "Gretna Green",
    },
    Station {
        code: "GER",
        name: "Gerrards Cross",
    },
    Station {
        code: "GFD",
        name: "Greenford",
    },
    Station {
        code: "GFF",
        name: "Gilfach Fargoed",
    },
    Station {
        code: "GFN",
        name: "Giffnock",
    },
    Station {
        code: "GGJ",
        name: "Georgemas Junction",
    },
    Station {
        code: "GGT",
        name: "Glasgow Airport",
    },
    Station {
        code: "GGV",
        name: "Gargrave",
    },
    Station {
        code: "GIG",
        name: "Giggleswick",
    },
    Station {
        code: "GIL",
        name: "Gillingham (Dorset)",
    },
    Station {
        code: "GIP",
        name: "Gipsy Hill",
    },
    Station {
        code: "GIR",
        name: "Girvan",
    },
    Station {
        code: "GKC",
        name: "Greenock Central",
    },
    Station {
        code: "GKW",
        name: "Greenock West",
    },
    Station {
        code: "GLC",
        name: "Glasgow Central",
    },
    Station {
        code: "GLD",
        name: "Guildford",
    },
    Station {
        code: "GLE",
        name: "Gleneagles",
    },
    Station {
        code: "GLF",
        name: "Glenfinnan",
    },
    Station {
        code: "GLG",
        name: "Glengarnock",
    },
    Station {
        code: "GLH",
        name: "Glasshoughton",
    },
    Station {
        code: "GLM",
        name: "Gillingham (Kent)",
    },
    Station {
        code: "GLO",
        name: "Glossop",
    },
    Station {
        code: "GLQ",
        name: "Glasgow Queen Street",
    },
    Station {
        code: "GLS",
        name: "Glaisdale",
    },
    Station {
        code: "GLT",
        name: "Glenrothes",
    },
    Station {
        code: "GLY",
        name: "Glynde",
    },
    Station {
        code: "GLZ",
        name: "Glazebrook",
    },
    Station {
        code: "GMB",
        name: "Grimsby Town",
    },
    Station {
        code: "GMD",
        name: "Grimsby Docks",
    },
    Station {
        code: "GMG",
        name: "Garth (Bridgend)",
    },
    Station {
        code: "GMN",
        name: "Great Missenden",
    },
    Station {
        code: "GMT",
        name: "Grosmont",
    },
    Station {
        code: "GMV",
        name: "Great Malvern",
    },
    Station {
        code: "GMY",
        name: "Goodmayes",
    },
    Station {
        code: "GNB",
        name: "Gainsborough Central",
    },
    Station {
        code: "GNF",
        name: "Greenfield",
    },
    Station {
        code: "GNH",
        name: "Greenhithe",
    },
    Station {
        code: "GNL",
        name: "Green Lane",
    },
    Station {
        code: "GNR",
        name: "Green Road",
    },
    Station {
        code: "GNT",
        name: "Gunton",
    },
    Station {
        code: "GNW",
        name: "Greenwich",
    },
    Station {
        code: "GOB",
        name: "Gobowen",
    },
    Station {
        code: "GOD",
        name: "Godalming",
    },
    Station {
        code: "GOE",
        name: "Goldthorpe",
    },
    Station {
        code: "GOF",
        name: "Golf Street",
    },
    Station {
        code: "GOL",
        name: "Golspie",
    },
    Station {
        code: "GOM",
        name: "Gomshall",
    },
    Station {
        code: "GOO",
        name: "Goole",
    },
    Station {
        code: "GOP",
        name: "Gosport Bus E1/E2",
    },
    Station {
        code: "GOR",
        name: "Goring & Streatley",
    },
    Station {
        code: "GOS",
        name: "Grange Over Sands",
    },
    Station {
        code: "GOX",
        name: "Goxhill",
    },
    Station {
        code: "GOY",
        name: "Gorey       (CIV)",
    },
    Station {
        code: "GPK",
        name: "Grange Park",
    },
    Station {
        code: "GPO",
        name: "Gospel Oak",
    },
    Station {
        code: "GRA",
        name: "Grantham",
    },
    Station {
        code: "GRB",
        name: "Great Bentley",
    },
    Station {
        code: "GRC",
        name: "Great Chesterford",
    },
    Station {
        code: "GRF",
        name: "Garforth",
    },
    Station {
        code: "GRH",
        name: "Gartcosh",
    },
    Station {
        code: "GRK",
        name: "Gourock",
    },
    Station {
        code: "GRL",
        name: "Greenfaulds",
    },
    Station {
        code: "GRN",
        name: "Grindleford",
    },
    Station {
        code: "GRP",
        name: "Grove Park",
    },
    Station {
        code: "GRS",
        name: "Garscadden",
    },
    Station {
        code: "GRT",
        name: "Grateley",
    },
    Station {
        code: "GRV",
        name: "Gravesend",
    },
    Station {
        code: "GRY",
        name: "Grays",
    },
    Station {
        code: "GSC",
        name: "Gilshochill",
    },
    Station {
        code: "GSD",
        name: "Garsdale",
    },
    Station {
        code: "GSL",
        name: "Gunnislake",
    },
    Station {
        code: "GSN",
        name: "Garston (Herts)",
    },
    Station {
        code: "GSO",
        name: "Greystones  (CIV)",
    },
    Station {
        code: "GST",
        name: "Gathurst",
    },
    Station {
        code: "GSW",
        name: "Garswood",
    },
    Station {
        code: "GSY",
        name: "Guiseley",
    },
    Station {
        code: "GTA",
        name: "Great Ayton",
    },
    Station {
        code: "GTH",
        name: "Garth (Powys)",
    },
    Station {
        code: "GTN",
        name: "Grangetown (Cardiff)",
    },
    Station {
        code: "GTO",
        name: "Gorton",
    },
    Station {
        code: "GTR",
        name: "Goostrey",
    },
    Station {
        code: "GTW",
        name: "Gatwick Airport",
    },
    Station {
        code: "GTY",
        name: "Gatley",
    },
    Station {
        code: "GUD",
        name: "GOODRINGTON SANDS",
    },
    Station {
        code: "GUI",
        name: "Guide Bridge",
    },
    Station {
        code: "GUN",
        name: "Gunnersbury",
    },
    Station {
        code: "GUS",
        name: "Guernsey (Channel Islands)",
    },
    Station {
        code: "GVE",
        name: "Garve",
    },
    Station {
        code: "GVH",
        name: "Gravelly Hill",
    },
    Station {
        code: "GWE",
        name: "Gwersyllt",
    },
    Station {
        code: "GWN",
        name: "Gowerton",
    },
    Station {
        code: "GWY",
        name: "Galway      (CIV)",
    },
    Station {
        code: "GXX",
        name: "Gourock Pier",
    },
    Station {
        code: "GYM",
        name: "Great Yarmouth",
    },
    Station {
        code: "GYP",
        name: "Gypsy Lane",
    },
    Station {
        code: "HAB",
        name: "Habrough",
    },
    Station {
        code: "HAC",
        name: "Hackney Downs",
    },
    Station {
        code: "HAD",
        name: "Haddiscoe",
    },
    Station {
        code: "HAF",
        name: "Heathrow Term 4",
    },
    Station {
        code: "HAG",
        name: "Hagley",
    },
    Station {
        code: "HAI",
        name: "Halling",
    },
    Station {
        code: "HAL",
        name: "Hale",
    },
    Station {
        code: "HAM",
        name: "Hamworthy",
    },
    Station {
        code: "HAN",
        name: "Hanwell",
    },
    Station {
        code: "HAP",
        name: "Hatfield Peverel",
    },
    Station {
        code: "HAS",
        name: "Halesworth",
    },
    Station {
        code: "HAT",
        name: "Hatfield (Herts)",
    },
    Station {
        code: "HAV",
        name: "Havant",
    },
    Station {
        code: "HAY",
        name: "Hayes & Harlington",
    },
    Station {
        code: "HAZ",
        name: "Hazel Grove",
    },
    Station {
        code: "HBB",
        name: "Hubberts Bridge",
    },
    Station {
        code: "HBC",
        name: "Harbour City (Metrolink)",
    },
    Station {
        code: "HBD",
        name: "Hebden Bridge",
    },
    Station {
        code: "HBF",
        name: "Hever Brocasfarm",
    },
    Station {
        code: "HBL",
        name: "Headbolt Lane",
    },
    Station {
        code: "HBN",
        name: "Hollingbourne",
    },
    Station {
        code: "HBP",
        name: "Hornbeam Park",
    },
    Station {
        code: "HBY",
        name: "Hartlebury",
    },
    Station {
        code: "HCB",
        name: "Hackbridge",
    },
    Station {
        code: "HCH",
        name: "Holmes Chapel",
    },
    Station {
        code: "HCN",
        name: "Headcorn",
    },
    Station {
        code: "HCT",
        name: "Huncoat",
    },
    Station {
        code: "HDB",
        name: "Haydon Bridge",
    },
    Station {
        code: "HDE",
        name: "Hedge End",
    },
    Station {
        code: "HDF",
        name: "Hadfield",
    },
    Station {
        code: "HDG",
        name: "Heald Green",
    },
    Station {
        code: "HDH",
        name: "Hampstead Heath",
    },
    Station {
        code: "HDL",
        name: "Headstone Lane",
    },
    Station {
        code: "HDM",
        name: "Haddenham & T P'way",
    },
    Station {
        code: "HDN",
        name: "Harlesden",
    },
    Station {
        code: "HDW",
        name: "Hadley Wood",
    },
    Station {
        code: "HDY",
        name: "Headingley",
    },
    Station {
        code: "HEC",
        name: "Heckington",
    },
    Station {
        code: "HED",
        name: "Halewood",
    },
    Station {
        code: "HEI",
        name: "Heighington",
    },
    Station {
        code: "HEL",
        name: "Hensall",
    },
    Station {
        code: "HEN",
        name: "Hendon",
    },
    Station {
        code: "HER",
        name: "Hersham",
    },
    Station {
        code: "HES",
        name: "Hessle",
    },
    Station {
        code: "HEV",
        name: "Hever",
    },
    Station {
        code: "HEW",
        name: "Heworth",
    },
    Station {
        code: "HEX",
        name: "Hexham",
    },
    Station {
        code: "HFD",
        name: "Hereford",
    },
    Station {
        code: "HFE",
        name: "Hertford East",
    },
    Station {
        code: "HFN",
        name: "Hertford North",
    },
    Station {
        code: "HFS",
        name: "Hatfield & Stainforth",
    },
    Station {
        code: "HFX",
        name: "Halifax",
    },
    Station {
        code: "HGD",
        name: "Hungerford",
    },
    Station {
        code: "HGF",
        name: "Hag Fold",
    },
    Station {
        code: "HGG",
        name: "Haggerston",
    },
    Station {
        code: "HGM",
        name: "Higham (Kent)",
    },
    Station {
        code: "HGN",
        name: "Hough Green",
    },
    Station {
        code: "HGR",
        name: "Hither Green",
    },
    Station {
        code: "HGS",
        name: "Hastings",
    },
    Station {
        code: "HGT",
        name: "Harrogate",
    },
    Station {
        code: "HGY",
        name: "Harringay",
    },
    Station {
        code: "HHB",
        name: "Heysham Port",
    },
    Station {
        code: "HHD",
        name: "Holyhead",
    },
    Station {
        code: "HHE",
        name: "Haywards Heath",
    },
    Station {
        code: "HHL",
        name: "Heath High Level",
    },
    Station {
        code: "HHY",
        name: "Highbury & Islington",
    },
    Station {
        code: "HIA",
        name: "Hampton-in-Arden",
    },
    Station {
        code: "HIB",
        name: "High Brooms",
    },
    Station {
        code: "HID",
        name: "Hall-i'-Th'-Wood",
    },
    Station {
        code: "HIG",
        name: "Highbridge",
    },
    Station {
        code: "HIL",
        name: "Hillside",
    },
    Station {
        code: "HIN",
        name: "Hindley",
    },
    Station {
        code: "HIP",
        name: "Highams Park",
    },
    Station {
        code: "HIR",
        name: "Horton in Ribblesdale",
    },
    Station {
        code: "HIT",
        name: "Hitchin",
    },
    Station {
        code: "HKC",
        name: "Hackney Central",
    },
    Station {
        code: "HKH",
        name: "Hawkhead",
    },
    Station {
        code: "HKM",
        name: "Hykeham",
    },
    Station {
        code: "HKN",
        name: "Hucknall",
    },
    Station {
        code: "HKW",
        name: "Hackney Wick",
    },
    Station {
        code: "HLB",
        name: "Hildenborough",
    },
    Station {
        code: "HLC",
        name: "Helensburgh Central",
    },
    Station {
        code: "HLD",
        name: "Hellifield",
    },
    Station {
        code: "HLE",
        name: "Hillington East",
    },
    Station {
        code: "HLF",
        name: "Hillfoot",
    },
    Station {
        code: "HLG",
        name: "Hall Green",
    },
    Station {
        code: "HLI",
        name: "Healing",
    },
    Station {
        code: "HLL",
        name: "Heath Low Level",
    },
    Station {
        code: "HLM",
        name: "Holmwood (Surrey)",
    },
    Station {
        code: "HLN",
        name: "Harlington (Beds)",
    },
    Station {
        code: "HLR",
        name: "Hall Road",
    },
    Station {
        code: "HLS",
        name: "Hilsea",
    },
    Station {
        code: "HLU",
        name: "Helensburgh Upper",
    },
    Station {
        code: "HLW",
        name: "Hillington West",
    },
    Station {
        code: "HLY",
        name: "Holytown",
    },
    Station {
        code: "HMC",
        name: "Hampton Court",
    },
    Station {
        code: "HMD",
        name: "Hampden Park (Sussex)",
    },
    Station {
        code: "HME",
        name: "Hamble",
    },
    Station {
        code: "HMK",
        name: "Hawes",
    },
    Station {
        code: "HML",
        name: "Hemel Hempstead",
    },
    Station {
        code: "HMM",
        name: "Hammerton",
    },
    Station {
        code: "HMN",
        name: "Homerton",
    },
    Station {
        code: "HMP",
        name: "Hampton (London)",
    },
    Station {
        code: "HMS",
        name: "Helmsdale",
    },
    Station {
        code: "HMT",
        name: "Ham Street",
    },
    Station {
        code: "HMW",
        name: "Hampton Wick",
    },
    Station {
        code: "HMY",
        name: "Hairmyres",
    },
    Station {
        code: "HNA",
        name: "Hinton Admiral",
    },
    Station {
        code: "HNB",
        name: "Herne Bay",
    },
    Station {
        code: "HNC",
        name: "Hamilton Central",
    },
    Station {
        code: "HND",
        name: "Hanborough",
    },
    Station {
        code: "HNF",
        name: "Hednesford",
    },
    Station {
        code: "HNG",
        name: "Hengoed",
    },
    Station {
        code: "HNH",
        name: "Herne Hill",
    },
    Station {
        code: "HNK",
        name: "Hinckley (Leics)",
    },
    Station {
        code: "HNL",
        name: "Henley in Arden",
    },
    Station {
        code: "HNT",
        name: "Huntly",
    },
    Station {
        code: "HNW",
        name: "Hamilton West",
    },
    Station {
        code: "HNX",
        name: "Hunts Cross",
    },
    Station {
        code: "HNY",
        name: "Hanley Bus Station",
    },
    Station {
        code: "HOC",
        name: "Hockley",
    },
    Station {
        code: "HOD",
        name: "Hollinwood (Metrolink)",
    },
    Station {
        code: "HOH",
        name: "Harrow-on-the-Hill",
    },
    Station {
        code: "HOK",
        name: "Hook",
    },
    Station {
        code: "HOL",
        name: "Holton Heath",
    },
    Station {
        code: "HON",
        name: "Honiton",
    },
    Station {
        code: "HOO",
        name: "Hooton",
    },
    Station {
        code: "HOP",
        name: "Hope (Derbyshire)",
    },
    Station {
        code: "HOR",
        name: "Horley",
    },
    Station {
        code: "HOT",
        name: "Henley on Thames",
    },
    Station {
        code: "HOU",
        name: "Hounslow",
    },
    Station {
        code: "HOV",
        name: "Hove",
    },
    Station {
        code: "HOW",
        name: "Howden",
    },
    Station {
        code: "HOX",
        name: "Hoxton",
    },
    Station {
        code: "HOY",
        name: "Honley",
    },
    Station {
        code: "HOZ",
        name: "Howwood (Strathclyde)",
    },
    Station {
        code: "HPA",
        name: "Honor Oak Park",
    },
    Station {
        code: "HPD",
        name: "Harpenden",
    },
    Station {
        code: "HPE",
        name: "Hope (Flints)",
    },
    Station {
        code: "HPK",
        name: "Heaton Park (Metrolink)",
    },
    Station {
        code: "HPL",
        name: "Hartlepool",
    },
    Station {
        code: "HPN",
        name: "Hapton",
    },
    Station {
        code: "HPQ",
        name: "Harwich International",
    },
    Station {
        code: "HPT",
        name: "Hopton Heath",
    },
    Station {
        code: "HRD",
        name: "Harling Road",
    },
    Station {
        code: "HRE",
        name: "Horden",
    },
    Station {
        code: "HRH",
        name: "Horsham",
    },
    Station {
        code: "HRL",
        name: "Harlech",
    },
    Station {
        code: "HRM",
        name: "Harrietsham",
    },
    Station {
        code: "HRN",
        name: "Hornsey",
    },
    Station {
        code: "HRO",
        name: "Harold Wood",
    },
    Station {
        code: "HRR",
        name: "Harrington",
    },
    Station {
        code: "HRS",
        name: "Horsforth",
    },
    Station {
        code: "HRW",
        name: "Harrow & Wealdstone",
    },
    Station {
        code: "HRY",
        name: "Harringay Green Lanes",
    },
    Station {
        code: "HSB",
        name: "Helsby",
    },
    Station {
        code: "HSC",
        name: "Hoscar",
    },
    Station {
        code: "HSD",
        name: "Hamstead (Birmingham)",
    },
    Station {
        code: "HSG",
        name: "Hathersage",
    },
    Station {
        code: "HSK",
        name: "Hassocks",
    },
    Station {
        code: "HSL",
        name: "Haslemere",
    },
    Station {
        code: "HST",
        name: "High Street (Glasgow)",
    },
    Station {
        code: "HSW",
        name: "Heswall",
    },
    Station {
        code: "HSY",
        name: "Horsley",
    },
    Station {
        code: "HTC",
        name: "Heaton Chapel",
    },
    Station {
        code: "HTE",
        name: "Hatch End",
    },
    Station {
        code: "HTF",
        name: "Hartford",
    },
    Station {
        code: "HTH",
        name: "Handforth",
    },
    Station {
        code: "HTM",
        name: "Holt Town (Metrolink)",
    },
    Station {
        code: "HTN",
        name: "Hatton",
    },
    Station {
        code: "HTO",
        name: "Hightown",
    },
    Station {
        code: "HTR",
        name: "Heathrow Central Bus Station",
    },
    Station {
        code: "HTW",
        name: "Hartwood",
    },
    Station {
        code: "HTY",
        name: "Hattersley",
    },
    Station {
        code: "HUB",
        name: "Hunmanby",
    },
    Station {
        code: "HUD",
        name: "Huddersfield",
    },
    Station {
        code: "HUL",
        name: "Hull",
    },
    Station {
        code: "HUN",
        name: "Huntingdon",
    },
    Station {
        code: "HUP",
        name: "Humphrey Park",
    },
    Station {
        code: "HUR",
        name: "Hurst Green",
    },
    Station {
        code: "HUS",
        name: "Hunstanton Bus",
    },
    Station {
        code: "HUT",
        name: "Hutton Cranswick",
    },
    Station {
        code: "HUU",
        name: "Hull Bus Station",
    },
    Station {
        code: "HUY",
        name: "Huyton",
    },
    Station {
        code: "HVF",
        name: "Haverfordwest",
    },
    Station {
        code: "HVH",
        name: "HOEK VAN HOLLAND",
    },
    Station {
        code: "HVN",
        name: "Havenhouse",
    },
    Station {
        code: "HWA",
        name: "Heathrow Terminal 2 Bus",
    },
    Station {
        code: "HWB",
        name: "Hawarden Bridge",
    },
    Station {
        code: "HWC",
        name: "HARWICH TOWN",
    },
    Station {
        code: "HWD",
        name: "Hawarden",
    },
    Station {
        code: "HWE",
        name: "Heathrow Terminal 3 Bus",
    },
    Station {
        code: "HWF",
        name: "Heathrow Terminal 4 Bus",
    },
    Station {
        code: "HWH",
        name: "Haltwhistle",
    },
    Station {
        code: "HWI",
        name: "Horwich Parkway",
    },
    Station {
        code: "HWM",
        name: "Harlow Mill",
    },
    Station {
        code: "HWN",
        name: "Harlow Town",
    },
    Station {
        code: "HWV",
        name: "Heathrow Terminal 5",
    },
    Station {
        code: "HWW",
        name: "How Wood",
    },
    Station {
        code: "HWX",
        name: "Heathrow Terminal 5 Bus",
    },
    Station {
        code: "HWY",
        name: "High Wycombe",
    },
    Station {
        code: "HXM",
        name: "Hoveton & Wroxham",
    },
    Station {
        code: "HXX",
        name: "Heathrow Terminals 2 & 3",
    },
    Station {
        code: "HYB",
        name: "Honeybourne",
    },
    Station {
        code: "HYC",
        name: "Hyde Central",
    },
    Station {
        code: "HYD",
        name: "Heyford",
    },
    Station {
        code: "HYH",
        name: "Hythe (Essex)",
    },
    Station {
        code: "HYK",
        name: "Hoylake",
    },
    Station {
        code: "HYL",
        name: "Hayle",
    },
    Station {
        code: "HYM",
        name: "Haymarket",
    },
    Station {
        code: "HYN",
        name: "Hyndland",
    },
    Station {
        code: "HYR",
        name: "Haydons Road",
    },
    Station {
        code: "HYS",
        name: "Hayes (Kent)",
    },
    Station {
        code: "HYT",
        name: "Hyde North",
    },
    Station {
        code: "HYW",
        name: "Hinchley Wood",
    },
    Station {
        code: "IBS",
        name: "Inverness Bus Station",
    },
    Station {
        code: "IFD",
        name: "Ilford",
    },
    Station {
        code: "IFI",
        name: "Ifield",
    },
    Station {
        code: "IGD",
        name: "Invergordon",
    },
    Station {
        code: "ILK",
        name: "Ilkley",
    },
    Station {
        code: "ILN",
        name: "Ilkeston",
    },
    Station {
        code: "IMW",
        name: "Imperial Wharf",
    },
    Station {
        code: "INB",
        name: "Inverness Airport (Bus)",
    },
    Station {
        code: "INC",
        name: "Ince (Manchester)",
    },
    Station {
        code: "INE",
        name: "Ince & Elton",
    },
    Station {
        code: "ING",
        name: "Invergowrie",
    },
    Station {
        code: "INH",
        name: "Invershin",
    },
    Station {
        code: "INK",
        name: "Inverkeithing",
    },
    Station {
        code: "INP",
        name: "Inverkip",
    },
    Station {
        code: "INR",
        name: "Inverurie",
    },
    Station {
        code: "INS",
        name: "Insch",
    },
    Station {
        code: "INT",
        name: "Ingatestone",
    },
    Station {
        code: "INV",
        name: "Inverness",
    },
    Station {
        code: "IPS",
        name: "Ipswich",
    },
    Station {
        code: "IRL",
        name: "Irlam",
    },
    Station {
        code: "IRV",
        name: "Irvine",
    },
    Station {
        code: "ISL",
        name: "Isleworth",
    },
    Station {
        code: "ISP",
        name: "Islip",
    },
    Station {
        code: "IVA",
        name: "Inverness Airport",
    },
    Station {
        code: "IVR",
        name: "Iver",
    },
    Station {
        code: "IVY",
        name: "Ivybridge",
    },
    Station {
        code: "JCH",
        name: "James Cook Hospital",
    },
    Station {
        code: "JEQ",
        name: "Jewellery Quarter",
    },
    Station {
        code: "JHN",
        name: "Johnstone (Strathclyde)",
    },
    Station {
        code: "JOH",
        name: "Johnston (Pembrokeshire)",
    },
    Station {
        code: "JOR",
        name: "Jordanhill",
    },
    Station {
        code: "JSY",
        name: "Jersey (Channel Islands)",
    },
    Station {
        code: "KBC",
        name: "Kinbrace",
    },
    Station {
        code: "KBF",
        name: "Kirkby in Furness",
    },
    Station {
        code: "KBK",
        name: "Kents Bank",
    },
    Station {
        code: "KBM",
        name: "Kingsway Business Park (Metrolink)",
    },
    Station {
        code: "KBN",
        name: "Kilburn High Road",
    },
    Station {
        code: "KBW",
        name: "Knebworth",
    },
    Station {
        code: "KBX",
        name: "Kirby Cross",
    },
    Station {
        code: "KCK",
        name: "Knockholt",
    },
    Station {
        code: "KDB",
        name: "Kidbrooke",
    },
    Station {
        code: "KDG",
        name: "Kidsgrove",
    },
    Station {
        code: "KDR",
        name: "Kildare   (CIV)",
    },
    Station {
        code: "KDY",
        name: "Kirkcaldy",
    },
    Station {
        code: "KEB",
        name: "Kenley (A22)",
    },
    Station {
        code: "KEH",
        name: "Keith",
    },
    Station {
        code: "KEI",
        name: "Keighley",
    },
    Station {
        code: "KEL",
        name: "Kelvedon",
    },
    Station {
        code: "KEM",
        name: "Kemble",
    },
    Station {
        code: "KEN",
        name: "Kendal",
    },
    Station {
        code: "KET",
        name: "Kettering",
    },
    Station {
        code: "KEY",
        name: "Keyham",
    },
    Station {
        code: "KEZ",
        name: "Kettering Pb Bus",
    },
    Station {
        code: "KGE",
        name: "Kingsknowe",
    },
    Station {
        code: "KGH",
        name: "Kinghorn",
    },
    Station {
        code: "KGL",
        name: "Kings Langley",
    },
    Station {
        code: "KGM",
        name: "Kingham",
    },
    Station {
        code: "KGN",
        name: "Kings Nympton",
    },
    Station {
        code: "KGP",
        name: "Kings Park",
    },
    Station {
        code: "KGS",
        name: "Kings Sutton",
    },
    Station {
        code: "KGT",
        name: "Kilgetty",
    },
    Station {
        code: "KGX",
        name: "London Kings Cross",
    },
    Station {
        code: "KID",
        name: "Kidderminster",
    },
    Station {
        code: "KIL",
        name: "Kildonan",
    },
    Station {
        code: "KIN",
        name: "Kingussie",
    },
    Station {
        code: "KIR",
        name: "Kirkby (Merseyside)",
    },
    Station {
        code: "KIT",
        name: "Kintbury",
    },
    Station {
        code: "KIV",
        name: "Kiveton Bridge",
    },
    Station {
        code: "KKB",
        name: "Kirkby in Ashfield",
    },
    Station {
        code: "KKD",
        name: "Kirkdale",
    },
    Station {
        code: "KKH",
        name: "Kirkhill",
    },
    Station {
        code: "KKM",
        name: "Kirkham & Wesham",
    },
    Station {
        code: "KKN",
        name: "Kirknewton",
    },
    Station {
        code: "KKS",
        name: "Kirk Sandall",
    },
    Station {
        code: "KLB",
        name: "Kings Lynn Coach",
    },
    Station {
        code: "KLD",
        name: "Kildale",
    },
    Station {
        code: "KLF",
        name: "Kirkstall Forge",
    },
    Station {
        code: "KLL",
        name: "Killarney   (CIV)",
    },
    Station {
        code: "KLM",
        name: "Kilmaurs",
    },
    Station {
        code: "KLN",
        name: "Kings Lynn",
    },
    Station {
        code: "KLS",
        name: "KINGS LYNN BUSGN",
    },
    Station {
        code: "KLY",
        name: "Kenley",
    },
    Station {
        code: "KMH",
        name: "Kempston Hardwick",
    },
    Station {
        code: "KMK",
        name: "Kilmarnock",
    },
    Station {
        code: "KML",
        name: "Kemsley",
    },
    Station {
        code: "KMP",
        name: "Kempton Park",
    },
    Station {
        code: "KMS",
        name: "Kemsing",
    },
    Station {
        code: "KNA",
        name: "Knaresborough",
    },
    Station {
        code: "KND",
        name: "Kingswood",
    },
    Station {
        code: "KNE",
        name: "Kennett",
    },
    Station {
        code: "KNF",
        name: "Knutsford",
    },
    Station {
        code: "KNG",
        name: "Kingston",
    },
    Station {
        code: "KNI",
        name: "Knighton",
    },
    Station {
        code: "KNL",
        name: "Kensal Green",
    },
    Station {
        code: "KNN",
        name: "Kings Norton",
    },
    Station {
        code: "KNO",
        name: "Knottingley",
    },
    Station {
        code: "KNR",
        name: "Kensal Rise",
    },
    Station {
        code: "KNS",
        name: "Kennishead",
    },
    Station {
        code: "KNT",
        name: "Kenton",
    },
    Station {
        code: "KNU",
        name: "Knucklas",
    },
    Station {
        code: "KNW",
        name: "Kenilworth",
    },
    Station {
        code: "KNY",
        name: "Kilkenny    (CIV)",
    },
    Station {
        code: "KPA",
        name: "Kensington Olympia",
    },
    Station {
        code: "KPT",
        name: "Kilpatrick",
    },
    Station {
        code: "KRK",
        name: "Kirkconnel",
    },
    Station {
        code: "KSL",
        name: "Kearsley (Manchester)",
    },
    Station {
        code: "KSN",
        name: "Kearsney (Kent)",
    },
    Station {
        code: "KSW",
        name: "Kirkby Stephen",
    },
    Station {
        code: "KTH",
        name: "Kent House",
    },
    Station {
        code: "KTL",
        name: "Kirton Lindsey",
    },
    Station {
        code: "KTN",
        name: "Kentish Town",
    },
    Station {
        code: "KTR",
        name: "Kintore",
    },
    Station {
        code: "KTW",
        name: "Kentish Town West",
    },
    Station {
        code: "KVD",
        name: "Kelvindale",
    },
    Station {
        code: "KVP",
        name: "Kiveton Park",
    },
    Station {
        code: "KWB",
        name: "Kew Bridge",
    },
    Station {
        code: "KWD",
        name: "Kirkwood",
    },
    Station {
        code: "KWG",
        name: "Kew Gardens",
    },
    Station {
        code: "KWL",
        name: "Kidwelly",
    },
    Station {
        code: "KWN",
        name: "Kilwinning",
    },
    Station {
        code: "KWR",
        name: "Kingswear (for Dartmouth)",
    },
    Station {
        code: "KYL",
        name: "Kyle of Lochalsh",
    },
    Station {
        code: "KYN",
        name: "Keynsham",
    },
    Station {
        code: "LAC",
        name: "Lancing",
    },
    Station {
        code: "LAD",
        name: "Ladywell",
    },
    Station {
        code: "LAG",
        name: "Langwith-Whaley Thorns",
    },
    Station {
        code: "LAI",
        name: "Laindon",
    },
    Station {
        code: "LAK",
        name: "Lakenheath",
    },
    Station {
        code: "LAM",
        name: "Lamphey",
    },
    Station {
        code: "LAN",
        name: "Lancaster",
    },
    Station {
        code: "LAP",
        name: "Lapford",
    },
    Station {
        code: "LAR",
        name: "Largs",
    },
    Station {
        code: "LAS",
        name: "Llansamlet",
    },
    Station {
        code: "LAU",
        name: "Laurencekirk",
    },
    Station {
        code: "LAW",
        name: "Landywood",
    },
    Station {
        code: "LAY",
        name: "Layton (Lancs)",
    },
    Station {
        code: "LBG",
        name: "London Bridge",
    },
    Station {
        code: "LBK",
        name: "Long Buckby",
    },
    Station {
        code: "LBN",
        name: "Lisburn (N Ireland)",
    },
    Station {
        code: "LBO",
        name: "Loughborough",
    },
    Station {
        code: "LBR",
        name: "Llanbedr",
    },
    Station {
        code: "LBT",
        name: "Larbert",
    },
    Station {
        code: "LBZ",
        name: "Leighton Buzzard",
    },
    Station {
        code: "LCA",
        name: "LEEDS CASTLE",
    },
    Station {
        code: "LCB",
        name: "Lochboisdale",
    },
    Station {
        code: "LCC",
        name: "Lochluichart",
    },
    Station {
        code: "LCG",
        name: "Lochgelly",
    },
    Station {
        code: "LCK",
        name: "Lockwood",
    },
    Station {
        code: "LCL",
        name: "Lochailort",
    },
    Station {
        code: "LCN",
        name: "Lincoln",
    },
    Station {
        code: "LCS",
        name: "Locheilside",
    },
    Station {
        code: "LDN",
        name: "Llandanwg",
    },
    Station {
        code: "LDR",
        name: "Londonderry (N Ireland)",
    },
    Station {
        code: "LDS",
        name: "Leeds",
    },
    Station {
        code: "LDW",
        name: "Ladywell (Metrolink)",
    },
    Station {
        code: "LDY",
        name: "Ladybank",
    },
    Station {
        code: "LEA",
        name: "Leagrave",
    },
    Station {
        code: "LEB",
        name: "Lea Bridge",
    },
    Station {
        code: "LED",
        name: "Ledbury",
    },
    Station {
        code: "LEE",
        name: "Lee (London)",
    },
    Station {
        code: "LEG",
        name: "Lea Green",
    },
    Station {
        code: "LEH",
        name: "Lea Hall",
    },
    Station {
        code: "LEI",
        name: "Leicester",
    },
    Station {
        code: "LEL",
        name: "Lelant",
    },
    Station {
        code: "LEM",
        name: "Leyton Midland Road",
    },
    Station {
        code: "LEN",
        name: "Lenham",
    },
    Station {
        code: "LEO",
        name: "Leominster",
    },
    Station {
        code: "LER",
        name: "Leytonstone High Road",
    },
    Station {
        code: "LES",
        name: "Leigh-on-Sea",
    },
    Station {
        code: "LET",
        name: "Letchworth Garden City",
    },
    Station {
        code: "LEU",
        name: "Leuchars",
    },
    Station {
        code: "LEV",
        name: "LEVEN",
    },
    Station {
        code: "LEW",
        name: "Lewisham",
    },
    Station {
        code: "LEY",
        name: "Leyland",
    },
    Station {
        code: "LFD",
        name: "Lingfield",
    },
    Station {
        code: "LFL",
        name: "Leigh Fleur-de-Lis",
    },
    Station {
        code: "LFO",
        name: "Longford    (CIV)",
    },
    Station {
        code: "LGB",
        name: "Langbank",
    },
    Station {
        code: "LGD",
        name: "Lingwood",
    },
    Station {
        code: "LGE",
        name: "Long Eaton",
    },
    Station {
        code: "LGF",
        name: "Longfield",
    },
    Station {
        code: "LGG",
        name: "Langley Green",
    },
    Station {
        code: "LGJ",
        name: "Loughborough Junction",
    },
    Station {
        code: "LGK",
        name: "Longbeck",
    },
    Station {
        code: "LGM",
        name: "Langley Mill",
    },
    Station {
        code: "LGN",
        name: "Longton",
    },
    Station {
        code: "LGO",
        name: "Llangynllo",
    },
    Station {
        code: "LGS",
        name: "Langside",
    },
    Station {
        code: "LGW",
        name: "Langwathby",
    },
    Station {
        code: "LHA",
        name: "Loch Awe",
    },
    Station {
        code: "LHD",
        name: "Leatherhead",
    },
    Station {
        code: "LHE",
        name: "Loch Eil Outward Bound",
    },
    Station {
        code: "LHM",
        name: "Lealholm",
    },
    Station {
        code: "LHO",
        name: "Langho",
    },
    Station {
        code: "LHR",
        name: "Heathrow Bus",
    },
    Station {
        code: "LHS",
        name: "Limehouse",
    },
    Station {
        code: "LHW",
        name: "Lochwinnoch",
    },
    Station {
        code: "LIC",
        name: "Lichfield City",
    },
    Station {
        code: "LID",
        name: "Lidlington",
    },
    Station {
        code: "LIE",
        name: "Leiston (via Saxmundham)",
    },
    Station {
        code: "LIH",
        name: "Leigh (Kent)",
    },
    Station {
        code: "LIN",
        name: "Linlithgow",
    },
    Station {
        code: "LIP",
        name: "Liphook",
    },
    Station {
        code: "LIS",
        name: "Liss",
    },
    Station {
        code: "LIT",
        name: "Littlehampton",
    },
    Station {
        code: "LIV",
        name: "Liverpool Lime St",
    },
    Station {
        code: "LJL",
        name: "Lpool Airprt Bus",
    },
    Station {
        code: "LJN",
        name: "Limerick Junction (CIV)",
    },
    Station {
        code: "LKE",
        name: "Lake",
    },
    Station {
        code: "LLA",
        name: "Llanaber",
    },
    Station {
        code: "LLC",
        name: "Llandecwyn",
    },
    Station {
        code: "LLD",
        name: "Llandudno",
    },
    Station {
        code: "LLE",
        name: "Llanelli",
    },
    Station {
        code: "LLF",
        name: "Llanfairfechan",
    },
    Station {
        code: "LLG",
        name: "Llangadog",
    },
    Station {
        code: "LLH",
        name: "Llangennech",
    },
    Station {
        code: "LLI",
        name: "Llandybie",
    },
    Station {
        code: "LLJ",
        name: "Llandudno Junction",
    },
    Station {
        code: "LLL",
        name: "Llandeilo",
    },
    Station {
        code: "LLM",
        name: "Llangammarch",
    },
    Station {
        code: "LLN",
        name: "Llandaf",
    },
    Station {
        code: "LLO",
        name: "Llandrindod",
    },
    Station {
        code: "LLR",
        name: "Llanharan",
    },
    Station {
        code: "LLS",
        name: "Llanishen",
    },
    Station {
        code: "LLT",
        name: "Llanbister Road",
    },
    Station {
        code: "LLV",
        name: "Llandovery",
    },
    Station {
        code: "LLW",
        name: "Llwyngwril",
    },
    Station {
        code: "LLY",
        name: "Llwynypia",
    },
    Station {
        code: "LMN",
        name: "Long Marston",
    },
    Station {
        code: "LMR",
        name: "Low Moor",
    },
    Station {
        code: "LMS",
        name: "Leamington Spa",
    },
    Station {
        code: "LNB",
        name: "Llanbradach",
    },
    Station {
        code: "LND",
        name: "Longniddry",
    },
    Station {
        code: "LNE",
        name: "London International (for Eurostar)",
    },
    Station {
        code: "LNG",
        name: "Longcross",
    },
    Station {
        code: "LNK",
        name: "Lanark",
    },
    Station {
        code: "LNR",
        name: "Llanwrda",
    },
    Station {
        code: "LNW",
        name: "Llanwrtyd",
    },
    Station {
        code: "LNY",
        name: "Langley (Berks)",
    },
    Station {
        code: "LNZ",
        name: "Lenzie",
    },
    Station {
        code: "LOB",
        name: "Longbridge",
    },
    Station {
        code: "LOC",
        name: "Lockerbie",
    },
    Station {
        code: "LOE",
        name: "London Eurostar (CIV)",
    },
    Station {
        code: "LOF",
        name: "London Fields",
    },
    Station {
        code: "LOH",
        name: "Lostock Hall",
    },
    Station {
        code: "LOO",
        name: "Looe",
    },
    Station {
        code: "LOS",
        name: "Lostwithiel",
    },
    Station {
        code: "LOT",
        name: "Lostock",
    },
    Station {
        code: "LOW",
        name: "Lowdham",
    },
    Station {
        code: "LPD",
        name: "LUTON AIRPORT PARKWAY DART",
    },
    Station {
        code: "LPG",
        name: "Llanfairpwll",
    },
    Station {
        code: "LPO",
        name: "Lyme Regis via First Bus X51/X53",
    },
    Station {
        code: "LPR",
        name: "Long Preston",
    },
    Station {
        code: "LPT",
        name: "Longport",
    },
    Station {
        code: "LPW",
        name: "Lapworth",
    },
    Station {
        code: "LPY",
        name: "Liverpool South Parkway",
    },
    Station {
        code: "LRB",
        name: "London Road (Brighton)",
    },
    Station {
        code: "LRD",
        name: "London Road (Guildford)",
    },
    Station {
        code: "LRG",
        name: "Lairg",
    },
    Station {
        code: "LRH",
        name: "Larkhall",
    },
    Station {
        code: "LRK",
        name: "Limerick    (CIV)",
    },
    Station {
        code: "LRR",
        name: "London Road D R",
    },
    Station {
        code: "LSK",
        name: "Liskeard",
    },
    Station {
        code: "LSN",
        name: "Livingston North",
    },
    Station {
        code: "LST",
        name: "London Liverpool Street",
    },
    Station {
        code: "LSW",
        name: "Leasowe",
    },
    Station {
        code: "LSY",
        name: "Lower Sydenham",
    },
    Station {
        code: "LTG",
        name: "Lostock Gralam",
    },
    Station {
        code: "LTH",
        name: "Llanhilleth",
    },
    Station {
        code: "LTK",
        name: "Little Kimble",
    },
    Station {
        code: "LTL",
        name: "Littleborough",
    },
    Station {
        code: "LTM",
        name: "Lytham",
    },
    Station {
        code: "LTN",
        name: "Luton Airport Parkway",
    },
    Station {
        code: "LTP",
        name: "Littleport",
    },
    Station {
        code: "LTR",
        name: "Lampeter (Bus)",
    },
    Station {
        code: "LTS",
        name: "Lelant Saltings",
    },
    Station {
        code: "LTT",
        name: "Little Sutton",
    },
    Station {
        code: "LTV",
        name: "Lichfield Trent Valley",
    },
    Station {
        code: "LUA",
        name: "Luton Airport",
    },
    Station {
        code: "LUB",
        name: "Luton Bus",
    },
    Station {
        code: "LUD",
        name: "Ludlow",
    },
    Station {
        code: "LUT",
        name: "Luton",
    },
    Station {
        code: "LUX",
        name: "Luxulyan",
    },
    Station {
        code: "LVC",
        name: "Liverpool Central",
    },
    Station {
        code: "LVG",
        name: "Livingston South",
    },
    Station {
        code: "LVJ",
        name: "James St (Liverpool)",
    },
    Station {
        code: "LVM",
        name: "Levenshulme",
    },
    Station {
        code: "LVN",
        name: "Littlehaven",
    },
    Station {
        code: "LVS",
        name: "Liverpool Landing Stage",
    },
    Station {
        code: "LVT",
        name: "Lisvane & Thornhill",
    },
    Station {
        code: "LWH",
        name: "Lawrence Hill",
    },
    Station {
        code: "LWM",
        name: "Llantwit Major",
    },
    Station {
        code: "LWR",
        name: "Llanrwst",
    },
    Station {
        code: "LWS",
        name: "Lewes",
    },
    Station {
        code: "LWT",
        name: "Lowestoft",
    },
    Station {
        code: "LWY",
        name: "Langworthy (Metrolink)",
    },
    Station {
        code: "LYC",
        name: "Lympstone Commando",
    },
    Station {
        code: "LYD",
        name: "Lydney",
    },
    Station {
        code: "LYE",
        name: "Lye (West Midlands)",
    },
    Station {
        code: "LYM",
        name: "Lympstone Village",
    },
    Station {
        code: "LYP",
        name: "Lymington Pier",
    },
    Station {
        code: "LYT",
        name: "Lymington Town",
    },
    Station {
        code: "LZB",
        name: "Lazonby",
    },
    Station {
        code: "MAC",
        name: "Macclesfield",
    },
    Station {
        code: "MAE",
        name: "Maynooth    (CIV)",
    },
    Station {
        code: "MAG",
        name: "Maghull",
    },
    Station {
        code: "MAI",
        name: "Maidenhead",
    },
    Station {
        code: "MAJ",
        name: "Manulla Junction  (CIV)",
    },
    Station {
        code: "MAL",
        name: "Malden Manor",
    },
    Station {
        code: "MAN",
        name: "Manchester Piccadilly",
    },
    Station {
        code: "MAO",
        name: "Martins Heron",
    },
    Station {
        code: "MAR",
        name: "Margate",
    },
    Station {
        code: "MAS",
        name: "Manors",
    },
    Station {
        code: "MAT",
        name: "Matlock",
    },
    Station {
        code: "MAU",
        name: "Mauldeth Road",
    },
    Station {
        code: "MAW",
        name: "Mallow      (CIV)",
    },
    Station {
        code: "MAX",
        name: "Maxwell Park",
    },
    Station {
        code: "MAY",
        name: "Maybole",
    },
    Station {
        code: "MBH",
        name: "Muine Bheag (CIV)",
    },
    Station {
        code: "MBK",
        name: "Millbrook (Hants)",
    },
    Station {
        code: "MBR",
        name: "Middlesbrough",
    },
    Station {
        code: "MBT",
        name: "Marsh Barton",
    },
    Station {
        code: "MCB",
        name: "Moulsecoomb",
    },
    Station {
        code: "MCE",
        name: "Metrocentre",
    },
    Station {
        code: "MCF",
        name: "Martinscroft (Metrolink)",
    },
    Station {
        code: "MCH",
        name: "March",
    },
    Station {
        code: "MCM",
        name: "Morecambe",
    },
    Station {
        code: "MCN",
        name: "Machynlleth",
    },
    Station {
        code: "MCO",
        name: "Manchester Oxford Road",
    },
    Station {
        code: "MCT",
        name: "Media City (Metrolink)",
    },
    Station {
        code: "MCV",
        name: "Manchester Victoria",
    },
    Station {
        code: "MCZ",
        name: "Manchester (Central Zone)",
    },
    Station {
        code: "MDB",
        name: "Maidstone Barracks",
    },
    Station {
        code: "MDE",
        name: "Maidstone East",
    },
    Station {
        code: "MDG",
        name: "Midgham",
    },
    Station {
        code: "MDL",
        name: "Middlewood",
    },
    Station {
        code: "MDM",
        name: "Moor Road (Metrolink)",
    },
    Station {
        code: "MDN",
        name: "Maiden Newton",
    },
    Station {
        code: "MDS",
        name: "Morden South",
    },
    Station {
        code: "MDW",
        name: "Maidstone West",
    },
    Station {
        code: "MEC",
        name: "Meols Cop",
    },
    Station {
        code: "MEL",
        name: "Meldreth",
    },
    Station {
        code: "MEN",
        name: "Menheniot",
    },
    Station {
        code: "MEO",
        name: "Meols",
    },
    Station {
        code: "MEP",
        name: "Meopham",
    },
    Station {
        code: "MER",
        name: "Merthyr Tydfil",
    },
    Station {
        code: "MES",
        name: "Melton (Suffolk)",
    },
    Station {
        code: "MEV",
        name: "Merthyr Vale",
    },
    Station {
        code: "MEW",
        name: "Maesteg (Ewenny Road)",
    },
    Station {
        code: "MEX",
        name: "Mexborough",
    },
    Station {
        code: "MEY",
        name: "Merryton",
    },
    Station {
        code: "MFA",
        name: "Morfa Mawddach",
    },
    Station {
        code: "MFF",
        name: "Minffordd",
    },
    Station {
        code: "MFH",
        name: "Milford Haven",
    },
    Station {
        code: "MFL",
        name: "Mount Florida",
    },
    Station {
        code: "MFT",
        name: "Mansfield",
    },
    Station {
        code: "MGM",
        name: "Metheringham",
    },
    Station {
        code: "MGN",
        name: "Marston Green",
    },
    Station {
        code: "MHM",
        name: "Merstham",
    },
    Station {
        code: "MHR",
        name: "Market Harborough",
    },
    Station {
        code: "MHS",
        name: "Meadowhall",
    },
    Station {
        code: "MIA",
        name: "Manchester Airport",
    },
    Station {
        code: "MIC",
        name: "Micheldever",
    },
    Station {
        code: "MIE",
        name: "Millstreet  (CIV)",
    },
    Station {
        code: "MIH",
        name: "Mills Hill (Manchester)",
    },
    Station {
        code: "MIJ",
        name: "Mitcham Junction",
    },
    Station {
        code: "MIK",
        name: "Micklefield",
    },
    Station {
        code: "MIL",
        name: "Mill Hill Broadway",
    },
    Station {
        code: "MIM",
        name: "Moreton in Marsh",
    },
    Station {
        code: "MIN",
        name: "Milliken Park",
    },
    Station {
        code: "MIR",
        name: "Mirfield",
    },
    Station {
        code: "MIS",
        name: "Mistley",
    },
    Station {
        code: "MKC",
        name: "Milton Keynes Central",
    },
    Station {
        code: "MKM",
        name: "Melksham",
    },
    Station {
        code: "MKR",
        name: "Market Rasen",
    },
    Station {
        code: "MKT",
        name: "Marks Tey",
    },
    Station {
        code: "MLB",
        name: "Millbrook (Beds)",
    },
    Station {
        code: "MLD",
        name: "Mouldsworth",
    },
    Station {
        code: "MLF",
        name: "Milford (Surrey)",
    },
    Station {
        code: "MLG",
        name: "Mallaig",
    },
    Station {
        code: "MLH",
        name: "Mill Hill (Lancs)",
    },
    Station {
        code: "MLM",
        name: "Millom",
    },
    Station {
        code: "MLN",
        name: "Milngavie",
    },
    Station {
        code: "MLR",
        name: "Milnrow (Metrolink)",
    },
    Station {
        code: "MLS",
        name: "Melrose",
    },
    Station {
        code: "MLT",
        name: "Malton",
    },
    Station {
        code: "MLW",
        name: "Marlow",
    },
    Station {
        code: "MLY",
        name: "Morley",
    },
    Station {
        code: "MMO",
        name: "Melton Mowbray",
    },
    Station {
        code: "MNA",
        name: "Manchester Airport (Metrolink)",
    },
    Station {
        code: "MNC",
        name: "Markinch",
    },
    Station {
        code: "MNE",
        name: "Manea",
    },
    Station {
        code: "MNG",
        name: "Manningtree",
    },
    Station {
        code: "MNN",
        name: "Menston",
    },
    Station {
        code: "MNP",
        name: "Manor Park",
    },
    Station {
        code: "MNR",
        name: "Manor Road",
    },
    Station {
        code: "MNS",
        name: "Maghull North",
    },
    Station {
        code: "MOB",
        name: "Mobberley",
    },
    Station {
        code: "MOG",
        name: "Moorgate",
    },
    Station {
        code: "MOM",
        name: "Edgewtown   (CIV)",
    },
    Station {
        code: "MON",
        name: "Monifieth",
    },
    Station {
        code: "MOO",
        name: "Muir of Ord",
    },
    Station {
        code: "MOR",
        name: "Mortimer",
    },
    Station {
        code: "MOS",
        name: "Moss Side",
    },
    Station {
        code: "MOT",
        name: "Motspur Park",
    },
    Station {
        code: "MOZ",
        name: "Mold Bus",
    },
    Station {
        code: "MPK",
        name: "Mosspark",
    },
    Station {
        code: "MPL",
        name: "Marple",
    },
    Station {
        code: "MPT",
        name: "Morpeth",
    },
    Station {
        code: "MRB",
        name: "Manorbier",
    },
    Station {
        code: "MRD",
        name: "Morchard Road",
    },
    Station {
        code: "MRF",
        name: "Moorfields",
    },
    Station {
        code: "MRN",
        name: "Marden (Kent)",
    },
    Station {
        code: "MRP",
        name: "Moorthorpe",
    },
    Station {
        code: "MRR",
        name: "Morar",
    },
    Station {
        code: "MRS",
        name: "Monks Risborough",
    },
    Station {
        code: "MRT",
        name: "Moreton (Merseyside)",
    },
    Station {
        code: "MRW",
        name: "Meridian Water",
    },
    Station {
        code: "MRY",
        name: "Maryport",
    },
    Station {
        code: "MSD",
        name: "Moorside",
    },
    Station {
        code: "MSH",
        name: "Mossley Hill",
    },
    Station {
        code: "MSK",
        name: "Marske",
    },
    Station {
        code: "MSL",
        name: "Mossley (Manchester)",
    },
    Station {
        code: "MSM",
        name: "Monsall (Metrolink)",
    },
    Station {
        code: "MSN",
        name: "Marsden (Yorkshire)",
    },
    Station {
        code: "MSO",
        name: "Moston",
    },
    Station {
        code: "MSR",
        name: "Minster",
    },
    Station {
        code: "MSS",
        name: "Moses Gate",
    },
    Station {
        code: "MST",
        name: "Maesteg",
    },
    Station {
        code: "MSW",
        name: "Mansfield Woodhouse",
    },
    Station {
        code: "MTA",
        name: "Mountain Ash",
    },
    Station {
        code: "MTB",
        name: "Matlock Bath",
    },
    Station {
        code: "MTC",
        name: "Mitcham Eastfields",
    },
    Station {
        code: "MTE",
        name: "Mira Technology Park",
    },
    Station {
        code: "MTG",
        name: "Mottingham",
    },
    Station {
        code: "MTH",
        name: "Motherwell",
    },
    Station {
        code: "MTL",
        name: "Mortlake",
    },
    Station {
        code: "MTM",
        name: "Martin Mill",
    },
    Station {
        code: "MTN",
        name: "Moreton (Dorset)",
    },
    Station {
        code: "MTO",
        name: "Marton",
    },
    Station {
        code: "MTP",
        name: "Montpelier",
    },
    Station {
        code: "MTS",
        name: "Montrose",
    },
    Station {
        code: "MTV",
        name: "Mount Vernon",
    },
    Station {
        code: "MUB",
        name: "Musselburgh",
    },
    Station {
        code: "MUF",
        name: "Manchester United Football Ground",
    },
    Station {
        code: "MUI",
        name: "Muirend",
    },
    Station {
        code: "MUK",
        name: "MUCK (ISLE OF)",
    },
    Station {
        code: "MUL",
        name: "Mullingar   (CIV)",
    },
    Station {
        code: "MVL",
        name: "Malvern Link",
    },
    Station {
        code: "MYB",
        name: "London Marylebone",
    },
    Station {
        code: "MYH",
        name: "Maryhill",
    },
    Station {
        code: "MYL",
        name: "Maryland",
    },
    Station {
        code: "MYT",
        name: "Mytholmroyd",
    },
    Station {
        code: "MZH",
        name: "Maze Hill",
    },
    Station {
        code: "NAN",
        name: "Nantwich",
    },
    Station {
        code: "NAR",
        name: "Narberth",
    },
    Station {
        code: "NAY",
        name: "Newton Aycliffe",
    },
    Station {
        code: "NBA",
        name: "New Barnet",
    },
    Station {
        code: "NBC",
        name: "New Beckenham",
    },
    Station {
        code: "NBE",
        name: "Newbridge",
    },
    Station {
        code: "NBG",
        name: "Newbridge (CIV))",
    },
    Station {
        code: "NBM",
        name: "Newbold (Metrolink)",
    },
    Station {
        code: "NBN",
        name: "New Brighton",
    },
    Station {
        code: "NBR",
        name: "Narborough",
    },
    Station {
        code: "NBT",
        name: "Norbiton",
    },
    Station {
        code: "NBW",
        name: "North Berwick",
    },
    Station {
        code: "NBY",
        name: "Newbury",
    },
    Station {
        code: "NCE",
        name: "New Clee",
    },
    Station {
        code: "NCK",
        name: "New Cumnock",
    },
    Station {
        code: "NCL",
        name: "Newcastle",
    },
    Station {
        code: "NCM",
        name: "North Camp",
    },
    Station {
        code: "NCO",
        name: "Newcourt",
    },
    Station {
        code: "NCT",
        name: "Newark Castle",
    },
    Station {
        code: "NCZ",
        name: "Newcastle (Metro)",
    },
    Station {
        code: "NDL",
        name: "North Dulwich",
    },
    Station {
        code: "NEG",
        name: "Newtongrange",
    },
    Station {
        code: "NEH",
        name: "New Eltham",
    },
    Station {
        code: "NEI",
        name: "Neilston",
    },
    Station {
        code: "NEL",
        name: "Nelson",
    },
    Station {
        code: "NEM",
        name: "New Malden",
    },
    Station {
        code: "NEN",
        name: "Nenagh      (CIV)",
    },
    Station {
        code: "NES",
        name: "Neston",
    },
    Station {
        code: "NET",
        name: "Netherfield",
    },
    Station {
        code: "NEW",
        name: "Newcraighall",
    },
    Station {
        code: "NFA",
        name: "North Fambridge",
    },
    Station {
        code: "NFD",
        name: "Northfield",
    },
    Station {
        code: "NFL",
        name: "Northfleet",
    },
    Station {
        code: "NFN",
        name: "Nafferton",
    },
    Station {
        code: "NGT",
        name: "Newington",
    },
    Station {
        code: "NHD",
        name: "Nunhead",
    },
    Station {
        code: "NHE",
        name: "New Hythe",
    },
    Station {
        code: "NHL",
        name: "New Holland",
    },
    Station {
        code: "NHY",
        name: "Newhey (Metrolink)",
    },
    Station {
        code: "NIM",
        name: "New Islington (Metrolink)",
    },
    Station {
        code: "NIT",
        name: "Nitshill",
    },
    Station {
        code: "NLN",
        name: "New Lane",
    },
    Station {
        code: "NLR",
        name: "North Llanrwst",
    },
    Station {
        code: "NLS",
        name: "Nailsea & Backwell",
    },
    Station {
        code: "NLT",
        name: "Northolt Park",
    },
    Station {
        code: "NLW",
        name: "Newton Le Willows",
    },
    Station {
        code: "NMC",
        name: "New Mills Central",
    },
    Station {
        code: "NMK",
        name: "Newmarket",
    },
    Station {
        code: "NMM",
        name: "Newton Heath & Moston (Metrolink)",
    },
    Station {
        code: "NMN",
        name: "New Mills Newtown",
    },
    Station {
        code: "NMP",
        name: "Northampton",
    },
    Station {
        code: "NMR",
        name: "Northern Moor (Metrolink)",
    },
    Station {
        code: "NMT",
        name: "Needham Market",
    },
    Station {
        code: "NNG",
        name: "Newark Northgate",
    },
    Station {
        code: "NNN",
        name: "Nuneaton Bus Station",
    },
    Station {
        code: "NNP",
        name: "Ninian Park",
    },
    Station {
        code: "NNT",
        name: "Nunthorpe",
    },
    Station {
        code: "NOA",
        name: "Newton-on-Ayr",
    },
    Station {
        code: "NOP",
        name: "NORTHUMBERLAND PARK (T&W)",
    },
    Station {
        code: "NOR",
        name: "Normanton",
    },
    Station {
        code: "NOT",
        name: "Nottingham",
    },
    Station {
        code: "NPD",
        name: "New Pudsey",
    },
    Station {
        code: "NQU",
        name: "North Queensferry",
    },
    Station {
        code: "NQY",
        name: "Newquay",
    },
    Station {
        code: "NRB",
        name: "Norbury",
    },
    Station {
        code: "NRC",
        name: "Newbury Racecourse",
    },
    Station {
        code: "NRD",
        name: "North Road",
    },
    Station {
        code: "NRN",
        name: "Nairn",
    },
    Station {
        code: "NRT",
        name: "Nethertown",
    },
    Station {
        code: "NRW",
        name: "Norwich",
    },
    Station {
        code: "NSB",
        name: "Normans Bay",
    },
    Station {
        code: "NSD",
        name: "Newstead",
    },
    Station {
        code: "NSG",
        name: "New Southgate",
    },
    Station {
        code: "NSH",
        name: "North Sheen",
    },
    Station {
        code: "NTA",
        name: "Newton Abbot",
    },
    Station {
        code: "NTC",
        name: "Newton St Cyres",
    },
    Station {
        code: "NTH",
        name: "Neath",
    },
    Station {
        code: "NTL",
        name: "Netley",
    },
    Station {
        code: "NTN",
        name: "Newton (Lanarkshire)",
    },
    Station {
        code: "NTR",
        name: "Northallerton",
    },
    Station {
        code: "NUF",
        name: "Nutfield",
    },
    Station {
        code: "NUM",
        name: "Northumberland Park (London)",
    },
    Station {
        code: "NUN",
        name: "Nuneaton",
    },
    Station {
        code: "NUT",
        name: "Nutbourne",
    },
    Station {
        code: "NVH",
        name: "Newhaven Harbour",
    },
    Station {
        code: "NVM",
        name: "Newhaven Marine",
    },
    Station {
        code: "NVN",
        name: "Newhaven Town",
    },
    Station {
        code: "NVR",
        name: "Navigation Road",
    },
    Station {
        code: "NWA",
        name: "North Walsham",
    },
    Station {
        code: "NWB",
        name: "North Wembley",
    },
    Station {
        code: "NWD",
        name: "Norwood Junction",
    },
    Station {
        code: "NWE",
        name: "Newport (Essex)",
    },
    Station {
        code: "NWH",
        name: "NEWSHAM",
    },
    Station {
        code: "NWI",
        name: "Northwich",
    },
    Station {
        code: "NWM",
        name: "New Milton",
    },
    Station {
        code: "NWN",
        name: "Newton for Hyde",
    },
    Station {
        code: "NWP",
        name: "Newport (South Wales)",
    },
    Station {
        code: "NWR",
        name: "Newtonmore",
    },
    Station {
        code: "NWT",
        name: "Newtown (Powys)",
    },
    Station {
        code: "NWX",
        name: "New Cross",
    },
    Station {
        code: "NWY",
        name: "Newry (N Ireland)",
    },
    Station {
        code: "NXG",
        name: "New Cross Gate",
    },
    Station {
        code: "OBN",
        name: "Oban",
    },
    Station {
        code: "OCK",
        name: "Ockendon",
    },
    Station {
        code: "OCM",
        name: "Oldham Central (Metrolink)",
    },
    Station {
        code: "OHL",
        name: "Old Hill",
    },
    Station {
        code: "OKE",
        name: "Okehampton",
    },
    Station {
        code: "OKL",
        name: "Oakleigh Park",
    },
    Station {
        code: "OKM",
        name: "Oakham",
    },
    Station {
        code: "OKN",
        name: "Oakengates",
    },
    Station {
        code: "OKS",
        name: "Oldham King Street (Metrolink)",
    },
    Station {
        code: "OLD",
        name: "Old Street",
    },
    Station {
        code: "OLF",
        name: "Oldfield Park",
    },
    Station {
        code: "OLM",
        name: "Oldham Mumps (Metrolink)",
    },
    Station {
        code: "OLT",
        name: "Olton",
    },
    Station {
        code: "OLY",
        name: "Ockley",
    },
    Station {
        code: "OMS",
        name: "Ormskirk",
    },
    Station {
        code: "OPK",
        name: "Orrell Park",
    },
    Station {
        code: "ORE",
        name: "Ore",
    },
    Station {
        code: "ORN",
        name: "Old Roan",
    },
    Station {
        code: "ORP",
        name: "Orpington",
    },
    Station {
        code: "ORR",
        name: "Orrell",
    },
    Station {
        code: "OTF",
        name: "Otford",
    },
    Station {
        code: "OTR",
        name: "Old Trafford (Metrolink)",
    },
    Station {
        code: "OUD",
        name: "Oundle Bus",
    },
    Station {
        code: "OUN",
        name: "Oulton Broad North",
    },
    Station {
        code: "OUS",
        name: "Oulton Broad South",
    },
    Station {
        code: "OUT",
        name: "Outwood",
    },
    Station {
        code: "OVE",
        name: "Overpool",
    },
    Station {
        code: "OVR",
        name: "Overton",
    },
    Station {
        code: "OXF",
        name: "Oxford",
    },
    Station {
        code: "OXN",
        name: "Oxenholme Lake District",
    },
    Station {
        code: "OXP",
        name: "Oxford Parkway",
    },
    Station {
        code: "OXS",
        name: "Oxshott",
    },
    Station {
        code: "OXT",
        name: "Oxted",
    },
    Station {
        code: "PAD",
        name: "London Paddington",
    },
    Station {
        code: "PAL",
        name: "Palmers Green",
    },
    Station {
        code: "PAN",
        name: "Pangbourne",
    },
    Station {
        code: "PAR",
        name: "Par",
    },
    Station {
        code: "PAT",
        name: "Patricroft",
    },
    Station {
        code: "PBL",
        name: "Parbold",
    },
    Station {
        code: "PBO",
        name: "Peterborough",
    },
    Station {
        code: "PBR",
        name: "Potters Bar",
    },
    Station {
        code: "PBU",
        name: "Peterbro Bus Stn",
    },
    Station {
        code: "PBY",
        name: "Pembrey & Burry Port",
    },
    Station {
        code: "PCD",
        name: "Pencoed",
    },
    Station {
        code: "PCN",
        name: "Paisley Canal",
    },
    Station {
        code: "PDG",
        name: "Padgate",
    },
    Station {
        code: "PDT",
        name: "Padstow Bus",
    },
    Station {
        code: "PDW",
        name: "Paddock Wood",
    },
    Station {
        code: "PEA",
        name: "Peartree",
    },
    Station {
        code: "PEB",
        name: "Pevensey Bay",
    },
    Station {
        code: "PEE",
        name: "Portree Bus",
    },
    Station {
        code: "PEG",
        name: "Pegswood",
    },
    Station {
        code: "PEM",
        name: "Pemberton",
    },
    Station {
        code: "PEN",
        name: "Penarth",
    },
    Station {
        code: "PER",
        name: "Penrhiwceiber",
    },
    Station {
        code: "PES",
        name: "Pensarn",
    },
    Station {
        code: "PET",
        name: "Petts Wood",
    },
    Station {
        code: "PEV",
        name: "Pevensey & Westham",
    },
    Station {
        code: "PEW",
        name: "Pewsey",
    },
    Station {
        code: "PFL",
        name: "Purfleet",
    },
    Station {
        code: "PFM",
        name: "Pontefract Monkhill",
    },
    Station {
        code: "PFR",
        name: "Pontefract Baghill",
    },
    Station {
        code: "PFT",
        name: "Poole Quay",
    },
    Station {
        code: "PFY",
        name: "Poulton Le Fylde",
    },
    Station {
        code: "PGM",
        name: "Pengam",
    },
    Station {
        code: "PGN",
        name: "Paignton",
    },
    Station {
        code: "PHG",
        name: "Penhelig",
    },
    Station {
        code: "PHM",
        name: "Peel Hall (Metrolink)",
    },
    Station {
        code: "PHR",
        name: "Penshurst",
    },
    Station {
        code: "PIA",
        name: "PILL",
    },
    Station {
        code: "PIL",
        name: "Pilning",
    },
    Station {
        code: "PIN",
        name: "Pinhoe",
    },
    Station {
        code: "PIT",
        name: "Pitlochry",
    },
    Station {
        code: "PIZ",
        name: "Pickering Bus",
    },
    Station {
        code: "PKG",
        name: "Penkridge",
    },
    Station {
        code: "PKS",
        name: "Parkstone (Dorset)",
    },
    Station {
        code: "PKT",
        name: "Park Street",
    },
    Station {
        code: "PLC",
        name: "Pluckley",
    },
    Station {
        code: "PLD",
        name: "Portslade",
    },
    Station {
        code: "PLE",
        name: "Pollokshields East",
    },
    Station {
        code: "PLG",
        name: "Polegate",
    },
    Station {
        code: "PLK",
        name: "Plockton",
    },
    Station {
        code: "PLM",
        name: "Plumley",
    },
    Station {
        code: "PLN",
        name: "Portlethen",
    },
    Station {
        code: "PLS",
        name: "Pleasington",
    },
    Station {
        code: "PLT",
        name: "Pontlottyn",
    },
    Station {
        code: "PLU",
        name: "Plumstead",
    },
    Station {
        code: "PLW",
        name: "Pollokshields West",
    },
    Station {
        code: "PLY",
        name: "Plymouth",
    },
    Station {
        code: "PMA",
        name: "Portsmouth Arms",
    },
    Station {
        code: "PMB",
        name: "Pembroke",
    },
    Station {
        code: "PMD",
        name: "Pembroke Dock",
    },
    Station {
        code: "PMH",
        name: "Portsmouth Harbour",
    },
    Station {
        code: "PMO",
        name: "Pomona (Metrolink)",
    },
    Station {
        code: "PMP",
        name: "Plumpton",
    },
    Station {
        code: "PMR",
        name: "Peckham Rye",
    },
    Station {
        code: "PMS",
        name: "Portsmouth & Southsea",
    },
    Station {
        code: "PMT",
        name: "Polmont",
    },
    Station {
        code: "PMW",
        name: "Penmaenmawr",
    },
    Station {
        code: "PNA",
        name: "Penally",
    },
    Station {
        code: "PNC",
        name: "Penychain",
    },
    Station {
        code: "PNE",
        name: "Penge East",
    },
    Station {
        code: "PNF",
        name: "Penyffordd",
    },
    Station {
        code: "PNL",
        name: "Pannal",
    },
    Station {
        code: "PNM",
        name: "Penmere",
    },
    Station {
        code: "PNQ",
        name: "Penzance Quay",
    },
    Station {
        code: "PNR",
        name: "Penrith",
    },
    Station {
        code: "PNS",
        name: "Penistone",
    },
    Station {
        code: "PNW",
        name: "Penge West",
    },
    Station {
        code: "PNY",
        name: "Pen Y Bont (Mid Wales)",
    },
    Station {
        code: "PNZ",
        name: "Penzance",
    },
    Station {
        code: "POH",
        name: "PORTISHEAD",
    },
    Station {
        code: "POK",
        name: "Pokesdown",
    },
    Station {
        code: "POL",
        name: "Polsloe Bridge",
    },
    Station {
        code: "PON",
        name: "Ponders End",
    },
    Station {
        code: "POO",
        name: "Poole",
    },
    Station {
        code: "POP",
        name: "Poppleton",
    },
    Station {
        code: "POR",
        name: "Porth",
    },
    Station {
        code: "POT",
        name: "Pontefract Tanshelf",
    },
    Station {
        code: "PPD",
        name: "Pontypridd",
    },
    Station {
        code: "PPK",
        name: "Possilpark",
    },
    Station {
        code: "PPL",
        name: "Pontypool & New Inn",
    },
    Station {
        code: "PPR",
        name: "Preston Prk Ldnr",
    },
    Station {
        code: "PRA",
        name: "Prestwick International",
    },
    Station {
        code: "PRB",
        name: "Prestbury",
    },
    Station {
        code: "PRE",
        name: "Preston (Lancs)",
    },
    Station {
        code: "PRH",
        name: "Penrhyndeudraeth",
    },
    Station {
        code: "PRI",
        name: "Portway Park & Ride",
    },
    Station {
        code: "PRL",
        name: "Prittlewell",
    },
    Station {
        code: "PRN",
        name: "Parton",
    },
    Station {
        code: "PRO",
        name: "Portarlington (CIV)",
    },
    Station {
        code: "PRP",
        name: "Preston Park",
    },
    Station {
        code: "PRR",
        name: "Princes Risborough",
    },
    Station {
        code: "PRS",
        name: "Prees",
    },
    Station {
        code: "PRT",
        name: "Prestatyn",
    },
    Station {
        code: "PRU",
        name: "Prudhoe",
    },
    Station {
        code: "PRW",
        name: "Perranwell",
    },
    Station {
        code: "PRY",
        name: "Perry Barr",
    },
    Station {
        code: "PSC",
        name: "Prescot",
    },
    Station {
        code: "PSE",
        name: "Pitsea",
    },
    Station {
        code: "PSH",
        name: "Pershore",
    },
    Station {
        code: "PSL",
        name: "Port Sunlight",
    },
    Station {
        code: "PSN",
        name: "Parson Street",
    },
    Station {
        code: "PST",
        name: "Prestonpans",
    },
    Station {
        code: "PSW",
        name: "Polesworth",
    },
    Station {
        code: "PTA",
        name: "Port Talbot Parkway",
    },
    Station {
        code: "PTB",
        name: "Pentre Bach",
    },
    Station {
        code: "PTC",
        name: "Portchester",
    },
    Station {
        code: "PTD",
        name: "Pontarddulais",
    },
    Station {
        code: "PTF",
        name: "Pantyffynnon",
    },
    Station {
        code: "PTG",
        name: "Port Glasgow",
    },
    Station {
        code: "PTH",
        name: "Perth",
    },
    Station {
        code: "PTK",
        name: "Partick",
    },
    Station {
        code: "PTL",
        name: "Priesthill & Darnley",
    },
    Station {
        code: "PTM",
        name: "Porthmadog",
    },
    Station {
        code: "PTN",
        name: "Portadown (N Ireland)",
    },
    Station {
        code: "PTO",
        name: "Portlaoise  (CIV)",
    },
    Station {
        code: "PTR",
        name: "Petersfield",
    },
    Station {
        code: "PTS",
        name: "Portrush (N Ireland)",
    },
    Station {
        code: "PTT",
        name: "Patterton",
    },
    Station {
        code: "PTW",
        name: "Prestwick (Strathclyde)",
    },
    Station {
        code: "PUL",
        name: "Pulborough",
    },
    Station {
        code: "PUO",
        name: "Purley Oaks",
    },
    Station {
        code: "PUR",
        name: "Purley",
    },
    Station {
        code: "PUT",
        name: "Putney",
    },
    Station {
        code: "PWC",
        name: "Prestwich (Metrolink)",
    },
    Station {
        code: "PWE",
        name: "Pollokshaws East",
    },
    Station {
        code: "PWL",
        name: "Pwllheli",
    },
    Station {
        code: "PWW",
        name: "Pollokshaws West",
    },
    Station {
        code: "PWY",
        name: "Patchway",
    },
    Station {
        code: "PYC",
        name: "Pontyclun",
    },
    Station {
        code: "PYE",
        name: "Pye Corner",
    },
    Station {
        code: "PYG",
        name: "Paisley Gilmour Street",
    },
    Station {
        code: "PYJ",
        name: "Paisley St James",
    },
    Station {
        code: "PYL",
        name: "Pyle",
    },
    Station {
        code: "PYN",
        name: "Penryn (Cornwall)",
    },
    Station {
        code: "PYP",
        name: "Pont-y-Pant",
    },
    Station {
        code: "PYT",
        name: "Poynton",
    },
    Station {
        code: "QAE",
        name: "Bristol Airport Bus",
    },
    Station {
        code: "QAF",
        name: "BRISTOL BUS AIR A3",
    },
    Station {
        code: "QBR",
        name: "Queenborough",
    },
    Station {
        code: "QCD",
        name: "CHILTERN DESTINATION",
    },
    Station {
        code: "QCF",
        name: "Countryfile Live",
    },
    Station {
        code: "QCO",
        name: "CHILTERN ORIGIN",
    },
    Station {
        code: "QDF",
        name: "Dean Forest Rly",
    },
    Station {
        code: "QDK",
        name: "Dereham Konectbus",
    },
    Station {
        code: "QDM",
        name: "Dereham (Coach)",
    },
    Station {
        code: "QED",
        name: "EMR DESTINATION",
    },
    Station {
        code: "QEO",
        name: "EMR ORIGIN",
    },
    Station {
        code: "QGL",
        name: "Golflink",
    },
    Station {
        code: "QGS",
        name: "Glasgow Subway",
    },
    Station {
        code: "QHC",
        name: "Hampton Court Flower Show",
    },
    Station {
        code: "QND",
        name: "NTH DESTINATION",
    },
    Station {
        code: "QNO",
        name: "NTH ORIGIN",
    },
    Station {
        code: "QPK",
        name: "Queens Park (Glasgow)",
    },
    Station {
        code: "QPW",
        name: "Queens Park (London)",
    },
    Station {
        code: "QRB",
        name: "Queenstown Road (Battersea)",
    },
    Station {
        code: "QRD",
        name: "Quainton Road",
    },
    Station {
        code: "QRP",
        name: "Queens Road Peckham",
    },
    Station {
        code: "QTD",
        name: "TRANSPENNINE DESTINATION",
    },
    Station {
        code: "QTO",
        name: "TRANSPENNINE ORIGIN",
    },
    Station {
        code: "QUI",
        name: "Quintrell Downs",
    },
    Station {
        code: "QXD",
        name: "XC Destination",
    },
    Station {
        code: "QXO",
        name: "XC Origin",
    },
    Station {
        code: "QYD",
        name: "Quakers Yard",
    },
    Station {
        code: "RAD",
        name: "Radley",
    },
    Station {
        code: "RAI",
        name: "Rainham (Kent)",
    },
    Station {
        code: "RAM",
        name: "Ramsgate",
    },
    Station {
        code: "RAN",
        name: "Rannoch",
    },
    Station {
        code: "RAU",
        name: "Rauceby",
    },
    Station {
        code: "RAV",
        name: "Ravenglass for Eskdale",
    },
    Station {
        code: "RAY",
        name: "Raynes Park",
    },
    Station {
        code: "RBR",
        name: "Robertsbridge",
    },
    Station {
        code: "RBU",
        name: "Reading Bus",
    },
    Station {
        code: "RCA",
        name: "Risca & Pontymister",
    },
    Station {
        code: "RCC",
        name: "Redcar Central",
    },
    Station {
        code: "RCD",
        name: "Rochdale",
    },
    Station {
        code: "RCE",
        name: "Redcar East",
    },
    Station {
        code: "RCF",
        name: "Radcliffe (Metrolink)",
    },
    Station {
        code: "RCM",
        name: "Roscommon   (CIV)",
    },
    Station {
        code: "RCR",
        name: "Roscrea     (CIV)",
    },
    Station {
        code: "RDA",
        name: "Redland",
    },
    Station {
        code: "RDB",
        name: "Redbridge (Hants)",
    },
    Station {
        code: "RDC",
        name: "Redditch",
    },
    Station {
        code: "RDD",
        name: "Riddlesdown",
    },
    Station {
        code: "RDF",
        name: "Radcliffe-on-Trent",
    },
    Station {
        code: "RDG",
        name: "Reading",
    },
    Station {
        code: "RDH",
        name: "Redhill",
    },
    Station {
        code: "RDM",
        name: "Riding Mill",
    },
    Station {
        code: "RDN",
        name: "Reddish North",
    },
    Station {
        code: "RDR",
        name: "Radyr",
    },
    Station {
        code: "RDS",
        name: "Reddish South",
    },
    Station {
        code: "RDT",
        name: "Radlett",
    },
    Station {
        code: "RDU",
        name: "Rathdrum    (CIV)",
    },
    Station {
        code: "RDW",
        name: "Reading West",
    },
    Station {
        code: "REB",
        name: "Romsey Bus",
    },
    Station {
        code: "REC",
        name: "Rectory Road",
    },
    Station {
        code: "RED",
        name: "Redruth",
    },
    Station {
        code: "REE",
        name: "Reedham (Norfolk)",
    },
    Station {
        code: "REI",
        name: "Reigate",
    },
    Station {
        code: "RET",
        name: "Retford",
    },
    Station {
        code: "RFD",
        name: "Rochford",
    },
    Station {
        code: "RFY",
        name: "Rock Ferry",
    },
    Station {
        code: "RGL",
        name: "Rugeley (Trent Valley)",
    },
    Station {
        code: "RGP",
        name: "Reading Green Park",
    },
    Station {
        code: "RGT",
        name: "Rugeley Town",
    },
    Station {
        code: "RGW",
        name: "Ramsgreave & Wilpshire",
    },
    Station {
        code: "RHA",
        name: "Doncaster Airport",
    },
    Station {
        code: "RHB",
        name: "Robin Hood Bay Bus",
    },
    Station {
        code: "RHD",
        name: "Ribblehead",
    },
    Station {
        code: "RHI",
        name: "Rhiwbina",
    },
    Station {
        code: "RHL",
        name: "Rhyl",
    },
    Station {
        code: "RHM",
        name: "Reedham (London)",
    },
    Station {
        code: "RHO",
        name: "Rhosneigr",
    },
    Station {
        code: "RHU",
        name: "RHUM (ISLE OF)",
    },
    Station {
        code: "RHY",
        name: "Rhymney",
    },
    Station {
        code: "RIA",
        name: "Rhoose (for Cardiff Airport)",
    },
    Station {
        code: "RIC",
        name: "Rickmansworth",
    },
    Station {
        code: "RID",
        name: "Ridgmont",
    },
    Station {
        code: "RIL",
        name: "Rice Lane",
    },
    Station {
        code: "RIS",
        name: "Rishton",
    },
    Station {
        code: "RKT",
        name: "Ruskington",
    },
    Station {
        code: "RLG",
        name: "Rayleigh",
    },
    Station {
        code: "RLN",
        name: "Rowlands Castle",
    },
    Station {
        code: "RMB",
        name: "Roman Bridge",
    },
    Station {
        code: "RMC",
        name: "Rotherham Central",
    },
    Station {
        code: "RMD",
        name: "Richmond (London)",
    },
    Station {
        code: "RMF",
        name: "Romford",
    },
    Station {
        code: "RMK",
        name: "Richmond Yks Bus",
    },
    Station {
        code: "RML",
        name: "Romiley",
    },
    Station {
        code: "RMR",
        name: "Rathmore (CIV)",
    },
    Station {
        code: "RNF",
        name: "Rainford",
    },
    Station {
        code: "RNH",
        name: "Rainhill",
    },
    Station {
        code: "RNM",
        name: "Rainham (Essex)",
    },
    Station {
        code: "RNR",
        name: "Roughton Road",
    },
    Station {
        code: "ROB",
        name: "Roby",
    },
    Station {
        code: "ROC",
        name: "Roche",
    },
    Station {
        code: "ROE",
        name: "Rotherhithe",
    },
    Station {
        code: "ROG",
        name: "Rogart",
    },
    Station {
        code: "ROL",
        name: "Rolleston",
    },
    Station {
        code: "ROM",
        name: "Romsey",
    },
    Station {
        code: "ROO",
        name: "Roose",
    },
    Station {
        code: "ROR",
        name: "Rogerstone",
    },
    Station {
        code: "ROS",
        name: "Rosyth",
    },
    Station {
        code: "ROW",
        name: "Rowley Regis",
    },
    Station {
        code: "RRB",
        name: "Ryder Brow",
    },
    Station {
        code: "RRM",
        name: "Robinswood Road (Metrolink)",
    },
    Station {
        code: "RRN",
        name: "Robroyston",
    },
    Station {
        code: "RSB",
        name: "Rosslare Europort",
    },
    Station {
        code: "RSG",
        name: "Rose Grove",
    },
    Station {
        code: "RSH",
        name: "Rose Hill",
    },
    Station {
        code: "RSN",
        name: "Reston",
    },
    Station {
        code: "RSS",
        name: "Rosslare Strand (CIV)",
    },
    Station {
        code: "RTC",
        name: "Rochdale Town Centre (Metrolink)",
    },
    Station {
        code: "RTH",
        name: "Roundthorn (Metrolink)",
    },
    Station {
        code: "RTN",
        name: "Renton",
    },
    Station {
        code: "RTR",
        name: "Rochester",
    },
    Station {
        code: "RTY",
        name: "Rothesay",
    },
    Station {
        code: "RUA",
        name: "Ruabon",
    },
    Station {
        code: "RUE",
        name: "Runcorn East",
    },
    Station {
        code: "RUF",
        name: "Rufford",
    },
    Station {
        code: "RUG",
        name: "Rugby",
    },
    Station {
        code: "RUN",
        name: "Runcorn",
    },
    Station {
        code: "RUS",
        name: "Ruswarp",
    },
    Station {
        code: "RUT",
        name: "Rutherglen",
    },
    Station {
        code: "RVB",
        name: "Ravensbourne",
    },
    Station {
        code: "RVN",
        name: "Ravensthorpe",
    },
    Station {
        code: "RWC",
        name: "Rawcliffe",
    },
    Station {
        code: "RYB",
        name: "Roy Bridge",
    },
    Station {
        code: "RYD",
        name: "Ryde Esplanade",
    },
    Station {
        code: "RYE",
        name: "Rye (Sussex)",
    },
    Station {
        code: "RYH",
        name: "Rye House",
    },
    Station {
        code: "RYN",
        name: "Roydon Essex",
    },
    Station {
        code: "RYP",
        name: "Ryde Pier Head",
    },
    Station {
        code: "RYR",
        name: "Ryde St Johns Road",
    },
    Station {
        code: "RYS",
        name: "Royston (Herts)",
    },
    Station {
        code: "SAA",
        name: "St Albans Abbey",
    },
    Station {
        code: "SAB",
        name: "Smallbrook Junction",
    },
    Station {
        code: "SAC",
        name: "St Albans City",
    },
    Station {
        code: "SAD",
        name: "Sandwell & Dudley",
    },
    Station {
        code: "SAE",
        name: "Saltaire",
    },
    Station {
        code: "SAF",
        name: "Salfords (Surrey)",
    },
    Station {
        code: "SAH",
        name: "Salhouse",
    },
    Station {
        code: "SAJ",
        name: "St Johns",
    },
    Station {
        code: "SAL",
        name: "Salisbury",
    },
    Station {
        code: "SAM",
        name: "Saltmarshe",
    },
    Station {
        code: "SAN",
        name: "Sandown",
    },
    Station {
        code: "SAO",
        name: "St Andrews Bus",
    },
    Station {
        code: "SAQ",
        name: "Salford Quays (Metrolink)",
    },
    Station {
        code: "SAR",
        name: "St Andrews Road",
    },
    Station {
        code: "SAS",
        name: "St Annes-on-the-Sea",
    },
    Station {
        code: "SAT",
        name: "South Acton",
    },
    Station {
        code: "SAU",
        name: "St Austell",
    },
    Station {
        code: "SAV",
        name: "Stratford upon Avon",
    },
    Station {
        code: "SAW",
        name: "Sawbridgeworth",
    },
    Station {
        code: "SAX",
        name: "Saxmundham",
    },
    Station {
        code: "SAY",
        name: "Swanley",
    },
    Station {
        code: "SBE",
        name: "Starbeck",
    },
    Station {
        code: "SBF",
        name: "St Budeaux Ferry Road",
    },
    Station {
        code: "SBJ",
        name: "Stourbridge Junction",
    },
    Station {
        code: "SBK",
        name: "South Bank",
    },
    Station {
        code: "SBM",
        name: "South Bermondsey",
    },
    Station {
        code: "SBP",
        name: "Stonebridge Park",
    },
    Station {
        code: "SBR",
        name: "Spean Bridge",
    },
    Station {
        code: "SBS",
        name: "St Bees",
    },
    Station {
        code: "SBT",
        name: "Stourbridge Town",
    },
    Station {
        code: "SBU",
        name: "Southbury",
    },
    Station {
        code: "SBV",
        name: "St Budeaux Victoria Road",
    },
    Station {
        code: "SBY",
        name: "Selby",
    },
    Station {
        code: "SCA",
        name: "Scarborough",
    },
    Station {
        code: "SCB",
        name: "Scrabster",
    },
    Station {
        code: "SCF",
        name: "Stechford",
    },
    Station {
        code: "SCG",
        name: "Stone Crossing",
    },
    Station {
        code: "SCH",
        name: "Scotstounhill",
    },
    Station {
        code: "SCM",
        name: "South Chadderton (Metrolink)",
    },
    Station {
        code: "SCN",
        name: "Stone Crown Street (Bus)",
    },
    Station {
        code: "SCR",
        name: "St Columb Road",
    },
    Station {
        code: "SCS",
        name: "Starcross",
    },
    Station {
        code: "SCT",
        name: "Scotscalder",
    },
    Station {
        code: "SCU",
        name: "Scunthorpe",
    },
    Station {
        code: "SCY",
        name: "South Croydon",
    },
    Station {
        code: "SDA",
        name: "Snodland",
    },
    Station {
        code: "SDB",
        name: "Sandbach",
    },
    Station {
        code: "SDC",
        name: "Shoreditch High Street",
    },
    Station {
        code: "SDE",
        name: "Shadwell",
    },
    Station {
        code: "SDF",
        name: "Saundersfoot",
    },
    Station {
        code: "SDG",
        name: "Sandling",
    },
    Station {
        code: "SDH",
        name: "Sudbury Hill",
    },
    Station {
        code: "SDI",
        name: "Stratford International CIV",
    },
    Station {
        code: "SDL",
        name: "Sandhills",
    },
    Station {
        code: "SDM",
        name: "Shieldmuir",
    },
    Station {
        code: "SDN",
        name: "St Denys",
    },
    Station {
        code: "SDP",
        name: "Sandplace",
    },
    Station {
        code: "SDR",
        name: "Saunderton",
    },
    Station {
        code: "SDT",
        name: "Sidmouth Bus",
    },
    Station {
        code: "SDW",
        name: "Sandwich",
    },
    Station {
        code: "SDY",
        name: "Sandy",
    },
    Station {
        code: "SEA",
        name: "Seaham",
    },
    Station {
        code: "SEB",
        name: "Seaburn",
    },
    Station {
        code: "SEC",
        name: "Seaton Carew",
    },
    Station {
        code: "SED",
        name: "Shelford (Cambs)",
    },
    Station {
        code: "SEE",
        name: "Southease",
    },
    Station {
        code: "SEF",
        name: "Seaford Sussex",
    },
    Station {
        code: "SEG",
        name: "Selling",
    },
    Station {
        code: "SEH",
        name: "Shoreham (Kent)",
    },
    Station {
        code: "SEJ",
        name: "SEATON DELAVAL",
    },
    Station {
        code: "SEL",
        name: "Sellafield",
    },
    Station {
        code: "SEM",
        name: "Seamer",
    },
    Station {
        code: "SEN",
        name: "Shenstone",
    },
    Station {
        code: "SER",
        name: "St Erth",
    },
    Station {
        code: "SES",
        name: "South Elmsall",
    },
    Station {
        code: "SET",
        name: "Settle",
    },
    Station {
        code: "SEV",
        name: "Sevenoaks",
    },
    Station {
        code: "SEZ",
        name: "Southease Church",
    },
    Station {
        code: "SFA",
        name: "Stratford International",
    },
    Station {
        code: "SFD",
        name: "Salford Central",
    },
    Station {
        code: "SFF",
        name: "Saffron Walden Bus",
    },
    Station {
        code: "SFI",
        name: "Shawfair",
    },
    Station {
        code: "SFL",
        name: "Seaforth & Litherland",
    },
    Station {
        code: "SFN",
        name: "Shifnal",
    },
    Station {
        code: "SFO",
        name: "Stanford-le-Hope",
    },
    Station {
        code: "SFR",
        name: "Shalford (Surrey)",
    },
    Station {
        code: "SFS",
        name: "Southfields (Underground)",
    },
    Station {
        code: "SGB",
        name: "Smethwick Galton Bridge",
    },
    Station {
        code: "SGE",
        name: "Swanage",
    },
    Station {
        code: "SGL",
        name: "South Gyle",
    },
    Station {
        code: "SGM",
        name: "St Germans",
    },
    Station {
        code: "SGN",
        name: "South Greenford",
    },
    Station {
        code: "SGQ",
        name: "Stone Grnvle Sq",
    },
    Station {
        code: "SGR",
        name: "Slade Green",
    },
    Station {
        code: "SHA",
        name: "Shaw & Crompton (Metrolink)",
    },
    Station {
        code: "SHB",
        name: "Shirebrook",
    },
    Station {
        code: "SHC",
        name: "Streethouse",
    },
    Station {
        code: "SHD",
        name: "Shildon",
    },
    Station {
        code: "SHE",
        name: "Sherborne",
    },
    Station {
        code: "SHF",
        name: "Sheffield",
    },
    Station {
        code: "SHH",
        name: "Shirehampton",
    },
    Station {
        code: "SHI",
        name: "Shiplake",
    },
    Station {
        code: "SHJ",
        name: "St Helens Junction",
    },
    Station {
        code: "SHL",
        name: "Shawlands",
    },
    Station {
        code: "SHM",
        name: "Sheringham",
    },
    Station {
        code: "SHN",
        name: "Shanklin",
    },
    Station {
        code: "SHO",
        name: "Sholing",
    },
    Station {
        code: "SHP",
        name: "Shepperton",
    },
    Station {
        code: "SHR",
        name: "Shrewsbury",
    },
    Station {
        code: "SHS",
        name: "Shotts",
    },
    Station {
        code: "SHT",
        name: "Shotton",
    },
    Station {
        code: "SHU",
        name: "Stonehouse",
    },
    Station {
        code: "SHV",
        name: "Southsea Hoverport",
    },
    Station {
        code: "SHW",
        name: "Shawford",
    },
    Station {
        code: "SHX",
        name: "SHOTTLE",
    },
    Station {
        code: "SHY",
        name: "Shipley (Yorkshire)",
    },
    Station {
        code: "SIA",
        name: "Southend Airport",
    },
    Station {
        code: "SIB",
        name: "Seahouses Bus",
    },
    Station {
        code: "SIC",
        name: "Silecroft",
    },
    Station {
        code: "SID",
        name: "Sidcup",
    },
    Station {
        code: "SIE",
        name: "Sherburn in Elmet",
    },
    Station {
        code: "SIH",
        name: "St Helier",
    },
    Station {
        code: "SIL",
        name: "Sileby",
    },
    Station {
        code: "SIN",
        name: "Singer",
    },
    Station {
        code: "SIP",
        name: "Shipton",
    },
    Station {
        code: "SIT",
        name: "Sittingbourne",
    },
    Station {
        code: "SIV",
        name: "St Ives (Cornwall)",
    },
    Station {
        code: "SJP",
        name: "St James Park (Exeter)",
    },
    Station {
        code: "SJS",
        name: "St James Street (Walthamstow)",
    },
    Station {
        code: "SKE",
        name: "Skewen",
    },
    Station {
        code: "SKG",
        name: "Skegness",
    },
    Station {
        code: "SKI",
        name: "Skipton",
    },
    Station {
        code: "SKM",
        name: "Stoke Mandeville",
    },
    Station {
        code: "SKN",
        name: "St Keyne",
    },
    Station {
        code: "SKS",
        name: "Stocksfield",
    },
    Station {
        code: "SKV",
        name: "St Keyne Village",
    },
    Station {
        code: "SKW",
        name: "Stoke Newington",
    },
    Station {
        code: "SLA",
        name: "Slateford",
    },
    Station {
        code: "SLB",
        name: "Saltburn",
    },
    Station {
        code: "SLD",
        name: "Salford Crescent",
    },
    Station {
        code: "SLE",
        name: "Sale (Metrolink)",
    },
    Station {
        code: "SLH",
        name: "Sleights",
    },
    Station {
        code: "SLI",
        name: "Sligo     (CIV)",
    },
    Station {
        code: "SLK",
        name: "Silkstone Common",
    },
    Station {
        code: "SLL",
        name: "Stallingborough",
    },
    Station {
        code: "SLO",
        name: "Slough",
    },
    Station {
        code: "SLQ",
        name: "St Leonards Warrior Square",
    },
    Station {
        code: "SLR",
        name: "Sleaford",
    },
    Station {
        code: "SLS",
        name: "Shettleston",
    },
    Station {
        code: "SLT",
        name: "Saltcoats",
    },
    Station {
        code: "SLV",
        name: "Silver Street",
    },
    Station {
        code: "SLW",
        name: "Salwick",
    },
    Station {
        code: "SLY",
        name: "Selly Oak",
    },
    Station {
        code: "SMA",
        name: "Small Heath",
    },
    Station {
        code: "SMB",
        name: "Smithy Bridge",
    },
    Station {
        code: "SMC",
        name: "Sampford Courtenay (closed)",
    },
    Station {
        code: "SMD",
        name: "Stamford (Lincs)",
    },
    Station {
        code: "SMG",
        name: "St Margarets (London)",
    },
    Station {
        code: "SMH",
        name: "Stamford Hill",
    },
    Station {
        code: "SMK",
        name: "Stowmarket",
    },
    Station {
        code: "SML",
        name: "Sea Mills",
    },
    Station {
        code: "SMM",
        name: "Shadowmoss (Metrolink)",
    },
    Station {
        code: "SMN",
        name: "Southminster",
    },
    Station {
        code: "SMO",
        name: "South Merton",
    },
    Station {
        code: "SMQ",
        name: "St Mary's Quay",
    },
    Station {
        code: "SMR",
        name: "Smethwick Rolfe Street",
    },
    Station {
        code: "SMT",
        name: "St Margarets (Herts)",
    },
    Station {
        code: "SMY",
        name: "St Mary Cray",
    },
    Station {
        code: "SNA",
        name: "Sandal & Agbrigg",
    },
    Station {
        code: "SND",
        name: "Sandhurst (Berks)",
    },
    Station {
        code: "SNE",
        name: "Stone (Staffs)",
    },
    Station {
        code: "SNF",
        name: "Shenfield",
    },
    Station {
        code: "SNG",
        name: "Sunningdale",
    },
    Station {
        code: "SNH",
        name: "St Helens Central",
    },
    Station {
        code: "SNI",
        name: "Snaith",
    },
    Station {
        code: "SNK",
        name: "Sankey",
    },
    Station {
        code: "SNL",
        name: "Stoneleigh",
    },
    Station {
        code: "SNN",
        name: "Swinton (Manchester)",
    },
    Station {
        code: "SNO",
        name: "St Neots",
    },
    Station {
        code: "SNR",
        name: "Sanderstead",
    },
    Station {
        code: "SNS",
        name: "Staines",
    },
    Station {
        code: "SNT",
        name: "Stanlow & Thornton",
    },
    Station {
        code: "SNW",
        name: "Swanwick",
    },
    Station {
        code: "SNY",
        name: "Sunnymeads",
    },
    Station {
        code: "SOA",
        name: "Southampton Airport Parkway",
    },
    Station {
        code: "SOB",
        name: "Southbourne",
    },
    Station {
        code: "SOC",
        name: "Southend Central",
    },
    Station {
        code: "SOE",
        name: "Southend East",
    },
    Station {
        code: "SOF",
        name: "South Woodham Ferrers",
    },
    Station {
        code: "SOG",
        name: "Stonegate",
    },
    Station {
        code: "SOH",
        name: "South Hampstead",
    },
    Station {
        code: "SOI",
        name: "Stow",
    },
    Station {
        code: "SOJ",
        name: "Soham",
    },
    Station {
        code: "SOK",
        name: "South Kenton",
    },
    Station {
        code: "SOL",
        name: "Solihull",
    },
    Station {
        code: "SOM",
        name: "South Milford",
    },
    Station {
        code: "SON",
        name: "Steeton & Silsden",
    },
    Station {
        code: "SOO",
        name: "Strood (Kent)",
    },
    Station {
        code: "SOP",
        name: "Southport",
    },
    Station {
        code: "SOR",
        name: "Sole Street",
    },
    Station {
        code: "SOS",
        name: "Stromness (Orkney)",
    },
    Station {
        code: "SOT",
        name: "Stoke on Trent",
    },
    Station {
        code: "SOU",
        name: "Southampton Central",
    },
    Station {
        code: "SOV",
        name: "Southend Victoria",
    },
    Station {
        code: "SOW",
        name: "Sowerby Bridge",
    },
    Station {
        code: "SOY",
        name: "Stornoway",
    },
    Station {
        code: "SPA",
        name: "Spalding",
    },
    Station {
        code: "SPB",
        name: "Shepherds Bush",
    },
    Station {
        code: "SPF",
        name: "Springfield",
    },
    Station {
        code: "SPH",
        name: "Shepherds Well",
    },
    Station {
        code: "SPI",
        name: "Spital",
    },
    Station {
        code: "SPK",
        name: "Sutton Parkway",
    },
    Station {
        code: "SPM",
        name: "Sale Water Park (Metrolink)",
    },
    Station {
        code: "SPN",
        name: "Spooner Row",
    },
    Station {
        code: "SPO",
        name: "Spondon",
    },
    Station {
        code: "SPP",
        name: "Shippea Hill",
    },
    Station {
        code: "SPR",
        name: "Springburn",
    },
    Station {
        code: "SPS",
        name: "Stepps",
    },
    Station {
        code: "SPT",
        name: "Stockport",
    },
    Station {
        code: "SPU",
        name: "Staplehurst",
    },
    Station {
        code: "SPX",
        name: "LONDN STPAN INTL",
    },
    Station {
        code: "SPY",
        name: "Shepley",
    },
    Station {
        code: "SQE",
        name: "Surrey Quays",
    },
    Station {
        code: "SQH",
        name: "Sanquhar",
    },
    Station {
        code: "SQU",
        name: "Squires Gate",
    },
    Station {
        code: "SRA",
        name: "Stratford (London)",
    },
    Station {
        code: "SRC",
        name: "Streatham Common",
    },
    Station {
        code: "SRD",
        name: "Stapleton Road",
    },
    Station {
        code: "SRF",
        name: "Stretford (Metrolink)",
    },
    Station {
        code: "SRG",
        name: "Seer Green & Jordans",
    },
    Station {
        code: "SRH",
        name: "Streatham Hill",
    },
    Station {
        code: "SRI",
        name: "Spring Road",
    },
    Station {
        code: "SRL",
        name: "Shirley (West Midlands)",
    },
    Station {
        code: "SRN",
        name: "Strines",
    },
    Station {
        code: "SRO",
        name: "Shireoaks",
    },
    Station {
        code: "SRR",
        name: "Sarn",
    },
    Station {
        code: "SRS",
        name: "Selhurst",
    },
    Station {
        code: "SRT",
        name: "Shortlands",
    },
    Station {
        code: "SRU",
        name: "South Ruislip",
    },
    Station {
        code: "SRY",
        name: "Shoeburyness",
    },
    Station {
        code: "SSA",
        name: "Southease A26",
    },
    Station {
        code: "SSC",
        name: "Seascale",
    },
    Station {
        code: "SSD",
        name: "Stansted Airport",
    },
    Station {
        code: "SSE",
        name: "Shoreham by Sea",
    },
    Station {
        code: "SSM",
        name: "Stocksmoor",
    },
    Station {
        code: "SSS",
        name: "Sheerness on Sea",
    },
    Station {
        code: "SST",
        name: "Stansted Mountfitchet",
    },
    Station {
        code: "STA",
        name: "Stafford",
    },
    Station {
        code: "STC",
        name: "Strathcarron",
    },
    Station {
        code: "STD",
        name: "Stroud (Glos)",
    },
    Station {
        code: "STE",
        name: "Streatham",
    },
    Station {
        code: "STF",
        name: "Stromeferry",
    },
    Station {
        code: "STG",
        name: "Stirling",
    },
    Station {
        code: "STH",
        name: "Shepreth",
    },
    Station {
        code: "STI",
        name: "Stadium of Light",
    },
    Station {
        code: "STJ",
        name: "Severn Tunnel Junction",
    },
    Station {
        code: "STK",
        name: "Stockton",
    },
    Station {
        code: "STL",
        name: "Southall",
    },
    Station {
        code: "STM",
        name: "St Michaels",
    },
    Station {
        code: "STN",
        name: "Stonehaven",
    },
    Station {
        code: "STO",
        name: "South Tottenham",
    },
    Station {
        code: "STP",
        name: "London St Pancras",
    },
    Station {
        code: "STQ",
        name: "Southampton Town Quay",
    },
    Station {
        code: "STR",
        name: "Stranraer",
    },
    Station {
        code: "STS",
        name: "Saltash",
    },
    Station {
        code: "STT",
        name: "Stewarton",
    },
    Station {
        code: "STU",
        name: "Sturry",
    },
    Station {
        code: "STV",
        name: "Stevenston",
    },
    Station {
        code: "STW",
        name: "Strawberry Hill",
    },
    Station {
        code: "STY",
        name: "Stratford-upon-Avon Parkway",
    },
    Station {
        code: "STZ",
        name: "St Peters",
    },
    Station {
        code: "SUC",
        name: "Sutton Common",
    },
    Station {
        code: "SUD",
        name: "Sudbury & Harrow Road",
    },
    Station {
        code: "SUG",
        name: "Sugar Loaf Halt",
    },
    Station {
        code: "SUM",
        name: "Summerston",
    },
    Station {
        code: "SUN",
        name: "Sunderland",
    },
    Station {
        code: "SUO",
        name: "Sutton (London)",
    },
    Station {
        code: "SUP",
        name: "Sundridge Park",
    },
    Station {
        code: "SUR",
        name: "Surbiton",
    },
    Station {
        code: "SUT",
        name: "Sutton Coldfield",
    },
    Station {
        code: "SUU",
        name: "Sunbury",
    },
    Station {
        code: "SUY",
        name: "Sudbury (Suffolk)",
    },
    Station {
        code: "SVB",
        name: "Severn Beach",
    },
    Station {
        code: "SVG",
        name: "Stevenage",
    },
    Station {
        code: "SVK",
        name: "Seven Kings",
    },
    Station {
        code: "SVL",
        name: "Staveley",
    },
    Station {
        code: "SVR",
        name: "Silverdale",
    },
    Station {
        code: "SVS",
        name: "Seven Sisters",
    },
    Station {
        code: "SWA",
        name: "Swansea",
    },
    Station {
        code: "SWB",
        name: "Swaffham (Coach)",
    },
    Station {
        code: "SWD",
        name: "Swinderby",
    },
    Station {
        code: "SWE",
        name: "Swineshead",
    },
    Station {
        code: "SWG",
        name: "Swaythling",
    },
    Station {
        code: "SWI",
        name: "Swindon (Wilts)",
    },
    Station {
        code: "SWJ",
        name: "St Werburgh's Road (Metrolink)",
    },
    Station {
        code: "SWK",
        name: "Southwick",
    },
    Station {
        code: "SWL",
        name: "Swale",
    },
    Station {
        code: "SWM",
        name: "Swanscombe",
    },
    Station {
        code: "SWN",
        name: "Swinton (South Yorks)",
    },
    Station {
        code: "SWO",
        name: "Snowdown",
    },
    Station {
        code: "SWR",
        name: "Stewartby",
    },
    Station {
        code: "SWS",
        name: "South Wigston",
    },
    Station {
        code: "SWT",
        name: "Slaithwaite",
    },
    Station {
        code: "SWY",
        name: "Sway",
    },
    Station {
        code: "SXY",
        name: "Saxilby",
    },
    Station {
        code: "SYA",
        name: "Styal",
    },
    Station {
        code: "SYB",
        name: "Stalybridge",
    },
    Station {
        code: "SYD",
        name: "Sydenham (London)",
    },
    Station {
        code: "SYH",
        name: "Sydenham Hill",
    },
    Station {
        code: "SYL",
        name: "Syon Lane",
    },
    Station {
        code: "SYS",
        name: "Syston",
    },
    Station {
        code: "SYT",
        name: "Somerleyton",
    },
    Station {
        code: "TAB",
        name: "Tame Bridge Parkway",
    },
    Station {
        code: "TAC",
        name: "Tackley",
    },
    Station {
        code: "TAD",
        name: "Tadworth",
    },
    Station {
        code: "TAF",
        name: "Taffs Well",
    },
    Station {
        code: "TAI",
        name: "Tain",
    },
    Station {
        code: "TAL",
        name: "Talsarnau",
    },
    Station {
        code: "TAM",
        name: "Tamworth",
    },
    Station {
        code: "TAP",
        name: "Taplow",
    },
    Station {
        code: "TAT",
        name: "Tattenham Corner",
    },
    Station {
        code: "TAU",
        name: "Taunton",
    },
    Station {
        code: "TAY",
        name: "Taynuilt",
    },
    Station {
        code: "TBD",
        name: "Three Bridges",
    },
    Station {
        code: "TBM",
        name: "Trafford Bar (Metrolink)",
    },
    Station {
        code: "TBR",
        name: "Tilbury Riverside",
    },
    Station {
        code: "TBW",
        name: "Tunbridge Wells",
    },
    Station {
        code: "TBY",
        name: "Thornaby",
    },
    Station {
        code: "TCR",
        name: "Tottenham Court Road (Elizabeth line)",
    },
    Station {
        code: "TDU",
        name: "Tondu",
    },
    Station {
        code: "TEA",
        name: "Teesside Airport",
    },
    Station {
        code: "TED",
        name: "Teddington",
    },
    Station {
        code: "TEE",
        name: "TIREE (ISLE OF)",
    },
    Station {
        code: "TEM",
        name: "Templemore  (CIV)",
    },
    Station {
        code: "TEN",
        name: "Tenby",
    },
    Station {
        code: "TEO",
        name: "Theobalds Grove",
    },
    Station {
        code: "TEY",
        name: "Teynham",
    },
    Station {
        code: "TFC",
        name: "Telford",
    },
    Station {
        code: "TGM",
        name: "Teignmouth",
    },
    Station {
        code: "TGS",
        name: "Ty Glas",
    },
    Station {
        code: "THA",
        name: "Thatcham",
    },
    Station {
        code: "THB",
        name: "Thornliebank",
    },
    Station {
        code: "THC",
        name: "Thurnscoe",
    },
    Station {
        code: "THD",
        name: "Thames Ditton",
    },
    Station {
        code: "THE",
        name: "Theale",
    },
    Station {
        code: "THH",
        name: "Thatto Heath",
    },
    Station {
        code: "THI",
        name: "Thirsk",
    },
    Station {
        code: "THL",
        name: "Tile Hill",
    },
    Station {
        code: "THM",
        name: "Thomastown  (CIV)",
    },
    Station {
        code: "THO",
        name: "Thornford",
    },
    Station {
        code: "THP",
        name: "Thanet Parkway",
    },
    Station {
        code: "THS",
        name: "Thurso",
    },
    Station {
        code: "THT",
        name: "Thorntonhall",
    },
    Station {
        code: "THU",
        name: "Thurgarton",
    },
    Station {
        code: "THW",
        name: "The Hawthorns",
    },
    Station {
        code: "TIL",
        name: "Tilbury Town",
    },
    Station {
        code: "TIM",
        name: "Timperley",
    },
    Station {
        code: "TIP",
        name: "Tipton",
    },
    Station {
        code: "TIR",
        name: "Tir Phil",
    },
    Station {
        code: "TIS",
        name: "Tisbury",
    },
    Station {
        code: "TLB",
        name: "Talybont",
    },
    Station {
        code: "TLC",
        name: "Tal-y-Cafn",
    },
    Station {
        code: "TLH",
        name: "Tilehurst",
    },
    Station {
        code: "TLK",
        name: "The Lakes",
    },
    Station {
        code: "TLS",
        name: "Thorpe-le-Soken",
    },
    Station {
        code: "TMC",
        name: "Templecombe",
    },
    Station {
        code: "TNA",
        name: "Thornton Abbey",
    },
    Station {
        code: "TNF",
        name: "Tonfanau",
    },
    Station {
        code: "TNN",
        name: "Thorne North",
    },
    Station {
        code: "TNP",
        name: "Tonypandy",
    },
    Station {
        code: "TNS",
        name: "Thorne South",
    },
    Station {
        code: "TOD",
        name: "Todmorden",
    },
    Station {
        code: "TOK",
        name: "Three Oaks",
    },
    Station {
        code: "TOL",
        name: "Tolworth",
    },
    Station {
        code: "TOM",
        name: "Tottenham Hale",
    },
    Station {
        code: "TON",
        name: "Tonbridge",
    },
    Station {
        code: "TOO",
        name: "Tooting",
    },
    Station {
        code: "TOP",
        name: "Topsham",
    },
    Station {
        code: "TOT",
        name: "Totnes",
    },
    Station {
        code: "TPB",
        name: "Thorpe Bay",
    },
    Station {
        code: "TPC",
        name: "Thorpe Culvert",
    },
    Station {
        code: "TPN",
        name: "Ton Pentre",
    },
    Station {
        code: "TPY",
        name: "Tipperary   (CIV)",
    },
    Station {
        code: "TQY",
        name: "Torquay",
    },
    Station {
        code: "TRA",
        name: "Trafford Park",
    },
    Station {
        code: "TRB",
        name: "Treherbert",
    },
    Station {
        code: "TRD",
        name: "Troed Y Rhiw",
    },
    Station {
        code: "TRE",
        name: "Trefforest Estate",
    },
    Station {
        code: "TRF",
        name: "Trefforest",
    },
    Station {
        code: "TRH",
        name: "Trehafod",
    },
    Station {
        code: "TRI",
        name: "Tring",
    },
    Station {
        code: "TRL",
        name: "Tralee      (CIV)",
    },
    Station {
        code: "TRM",
        name: "Trimley",
    },
    Station {
        code: "TRN",
        name: "Troon",
    },
    Station {
        code: "TRO",
        name: "Trowbridge",
    },
    Station {
        code: "TRR",
        name: "Torre",
    },
    Station {
        code: "TRS",
        name: "Thurston",
    },
    Station {
        code: "TRU",
        name: "Truro",
    },
    Station {
        code: "TRY",
        name: "Treorchy",
    },
    Station {
        code: "TTA",
        name: "Tadworth (Avenue)",
    },
    Station {
        code: "TTF",
        name: "Thetford",
    },
    Station {
        code: "TTH",
        name: "Thornton Heath",
    },
    Station {
        code: "TTN",
        name: "Totton",
    },
    Station {
        code: "TUH",
        name: "Tulse Hill",
    },
    Station {
        code: "TUL",
        name: "Tulloch",
    },
    Station {
        code: "TUM",
        name: "Tullamore   (CIV)",
    },
    Station {
        code: "TUR",
        name: "Turkey Street",
    },
    Station {
        code: "TUS",
        name: "Thurles     (CIV)",
    },
    Station {
        code: "TUT",
        name: "Tutbury & Hatton",
    },
    Station {
        code: "TVP",
        name: "Tiverton Parkway",
    },
    Station {
        code: "TWB",
        name: "Tweedbank",
    },
    Station {
        code: "TWI",
        name: "Twickenham",
    },
    Station {
        code: "TWN",
        name: "Town Green",
    },
    Station {
        code: "TWY",
        name: "Twyford",
    },
    Station {
        code: "TYC",
        name: "Ty Croes",
    },
    Station {
        code: "TYG",
        name: "Tygwyn",
    },
    Station {
        code: "TYL",
        name: "Tyndrum Lower",
    },
    Station {
        code: "TYS",
        name: "Tyseley",
    },
    Station {
        code: "TYW",
        name: "Tywyn",
    },
    Station {
        code: "UCK",
        name: "Uckfield",
    },
    Station {
        code: "UDD",
        name: "Uddingston",
    },
    Station {
        code: "UHA",
        name: "Uphall",
    },
    Station {
        code: "UHL",
        name: "Upper Holloway",
    },
    Station {
        code: "UIG",
        name: "Uig Bus",
    },
    Station {
        code: "ULC",
        name: "Ulceby",
    },
    Station {
        code: "ULL",
        name: "Ulleskelf",
    },
    Station {
        code: "ULP",
        name: "Ullapool",
    },
    Station {
        code: "ULV",
        name: "Ulverston",
    },
    Station {
        code: "UMB",
        name: "Umberleigh",
    },
    Station {
        code: "UNI",
        name: "University (Birmingham)",
    },
    Station {
        code: "UPH",
        name: "Upper Halliford",
    },
    Station {
        code: "UPL",
        name: "Upholland",
    },
    Station {
        code: "UPM",
        name: "Upminster",
    },
    Station {
        code: "UPT",
        name: "Upton (Merseyside)",
    },
    Station {
        code: "UPW",
        name: "Upwey",
    },
    Station {
        code: "URM",
        name: "Urmston",
    },
    Station {
        code: "UTT",
        name: "Uttoxeter",
    },
    Station {
        code: "UTY",
        name: "Upper Tyndrum",
    },
    Station {
        code: "UWL",
        name: "Upper Warlingham",
    },
    Station {
        code: "VAL",
        name: "Valley",
    },
    Station {
        code: "VIC",
        name: "London Victoria",
    },
    Station {
        code: "VIR",
        name: "Virginia Water",
    },
    Station {
        code: "VPM",
        name: "Velopark (Metrolink)",
    },
    Station {
        code: "VXH",
        name: "Vauxhall",
    },
    Station {
        code: "WAC",
        name: "Warrington Central",
    },
    Station {
        code: "WAD",
        name: "Wadhurst",
    },
    Station {
        code: "WAE",
        name: "London Waterloo East",
    },
    Station {
        code: "WAF",
        name: "Wallyford",
    },
    Station {
        code: "WAL",
        name: "Walton on Thames",
    },
    Station {
        code: "WAM",
        name: "Walmer",
    },
    Station {
        code: "WAN",
        name: "Wanborough",
    },
    Station {
        code: "WAO",
        name: "Walton (Merseyside)",
    },
    Station {
        code: "WAR",
        name: "Ware (Herts)",
    },
    Station {
        code: "WAS",
        name: "Watton-at-Stone",
    },
    Station {
        code: "WAT",
        name: "London Waterloo",
    },
    Station {
        code: "WAV",
        name: "Wavertree Technology Park",
    },
    Station {
        code: "WAW",
        name: "Warrington West",
    },
    Station {
        code: "WBC",
        name: "Waterbeach",
    },
    Station {
        code: "WBD",
        name: "Whitley Bridge",
    },
    Station {
        code: "WBE",
        name: "Wadebridge Bus",
    },
    Station {
        code: "WBL",
        name: "Warblington",
    },
    Station {
        code: "WBO",
        name: "Wimbledon Chase",
    },
    Station {
        code: "WBP",
        name: "West Brompton",
    },
    Station {
        code: "WBQ",
        name: "Warrington Bank Quay",
    },
    Station {
        code: "WBR",
        name: "Whaley Bridge",
    },
    Station {
        code: "WBY",
        name: "West Byfleet",
    },
    Station {
        code: "WCB",
        name: "Westcombe Park",
    },
    Station {
        code: "WCF",
        name: "Westcliff",
    },
    Station {
        code: "WCH",
        name: "Whitchurch (Hants)",
    },
    Station {
        code: "WCK",
        name: "Wick",
    },
    Station {
        code: "WCL",
        name: "West Calder",
    },
    Station {
        code: "WCM",
        name: "Wickham Market",
    },
    Station {
        code: "WCP",
        name: "Worcester Park",
    },
    Station {
        code: "WCR",
        name: "Whitecraigs",
    },
    Station {
        code: "WCT",
        name: "Watchet Bus",
    },
    Station {
        code: "WCX",
        name: "Wembley Stadium",
    },
    Station {
        code: "WCY",
        name: "West Croydon",
    },
    Station {
        code: "WDB",
        name: "Woodbridge",
    },
    Station {
        code: "WDD",
        name: "Widdrington",
    },
    Station {
        code: "WDE",
        name: "Wood End",
    },
    Station {
        code: "WDH",
        name: "Woodhouse",
    },
    Station {
        code: "WDI",
        name: "West Didsbury (Metrolink)",
    },
    Station {
        code: "WDL",
        name: "Woodhall",
    },
    Station {
        code: "WDM",
        name: "Windermere",
    },
    Station {
        code: "WDN",
        name: "Walsden",
    },
    Station {
        code: "WDO",
        name: "Waddon",
    },
    Station {
        code: "WDR",
        name: "Woodlands Road (Metrolink)",
    },
    Station {
        code: "WDS",
        name: "Woodlesford",
    },
    Station {
        code: "WDT",
        name: "West Drayton",
    },
    Station {
        code: "WDU",
        name: "West Dulwich",
    },
    Station {
        code: "WEA",
        name: "West Ealing",
    },
    Station {
        code: "WED",
        name: "Wedgwood",
    },
    Station {
        code: "WEE",
        name: "Weeley",
    },
    Station {
        code: "WEH",
        name: "West Ham",
    },
    Station {
        code: "WEL",
        name: "Wellingborough",
    },
    Station {
        code: "WEM",
        name: "Wem",
    },
    Station {
        code: "WEO",
        name: "Wedgwood Old Road",
    },
    Station {
        code: "WER",
        name: "Wedgwood Lane",
    },
    Station {
        code: "WES",
        name: "Westerton",
    },
    Station {
        code: "WET",
        name: "Weeton",
    },
    Station {
        code: "WEY",
        name: "Weymouth",
    },
    Station {
        code: "WFD",
        name: "Waterford   (CIV)",
    },
    Station {
        code: "WFF",
        name: "Whifflet",
    },
    Station {
        code: "WFH",
        name: "Watford High Street",
    },
    Station {
        code: "WFI",
        name: "Westerfield",
    },
    Station {
        code: "WFJ",
        name: "Watford Junction",
    },
    Station {
        code: "WFL",
        name: "Wainfleet",
    },
    Station {
        code: "WFN",
        name: "Watford North",
    },
    Station {
        code: "WGA",
        name: "Westgate on Sea",
    },
    Station {
        code: "WGC",
        name: "Welwyn Garden City",
    },
    Station {
        code: "WGN",
        name: "Wigan North Western",
    },
    Station {
        code: "WGR",
        name: "Woodgrange Park",
    },
    Station {
        code: "WGT",
        name: "Wigton",
    },
    Station {
        code: "WGV",
        name: "Wargrave",
    },
    Station {
        code: "WGW",
        name: "Wigan Wallgate",
    },
    Station {
        code: "WHA",
        name: "Westenhanger",
    },
    Station {
        code: "WHC",
        name: "Walthamstow Central",
    },
    Station {
        code: "WHD",
        name: "West Hampstead",
    },
    Station {
        code: "WHE",
        name: "Whalley (Lancs)",
    },
    Station {
        code: "WHG",
        name: "Westhoughton",
    },
    Station {
        code: "WHI",
        name: "Whitstable",
    },
    Station {
        code: "WHK",
        name: "Wythenshawe Town Centre (Metrolink)",
    },
    Station {
        code: "WHL",
        name: "White Hart Lane",
    },
    Station {
        code: "WHM",
        name: "Whimple",
    },
    Station {
        code: "WHN",
        name: "Whiston",
    },
    Station {
        code: "WHP",
        name: "West Hampstead Thameslink",
    },
    Station {
        code: "WHR",
        name: "West Horndon",
    },
    Station {
        code: "WHS",
        name: "Whyteleafe South",
    },
    Station {
        code: "WHT",
        name: "Whitchurch (Cardiff)",
    },
    Station {
        code: "WHY",
        name: "Whyteleafe",
    },
    Station {
        code: "WIC",
        name: "Wickford",
    },
    Station {
        code: "WID",
        name: "Widnes",
    },
    Station {
        code: "WIH",
        name: "Winchmore Hill",
    },
    Station {
        code: "WIJ",
        name: "Willesden Junction",
    },
    Station {
        code: "WIL",
        name: "Willington",
    },
    Station {
        code: "WIM",
        name: "Wimbledon",
    },
    Station {
        code: "WIN",
        name: "Winchester",
    },
    Station {
        code: "WIR",
        name: "WIRKSWORTH",
    },
    Station {
        code: "WIS",
        name: "Wisbech (Coach)",
    },
    Station {
        code: "WIV",
        name: "Wivenhoe",
    },
    Station {
        code: "WKB",
        name: "West Kilbride",
    },
    Station {
        code: "WKD",
        name: "Walkden",
    },
    Station {
        code: "WKF",
        name: "Wakefield Westgate",
    },
    Station {
        code: "WKG",
        name: "Workington",
    },
    Station {
        code: "WKI",
        name: "West Kirby",
    },
    Station {
        code: "WKK",
        name: "Wakefield Kirkgate",
    },
    Station {
        code: "WKL",
        name: "Wicklow   (CIV)",
    },
    Station {
        code: "WKM",
        name: "Wokingham",
    },
    Station {
        code: "WLA",
        name: "Woodlawn    (CIV)",
    },
    Station {
        code: "WLC",
        name: "Waltham Cross",
    },
    Station {
        code: "WLD",
        name: "West St Leonards",
    },
    Station {
        code: "WLE",
        name: "Whittlesea",
    },
    Station {
        code: "WLF",
        name: "Whittlesford Parkway",
    },
    Station {
        code: "WLG",
        name: "Wallasey Grove Road",
    },
    Station {
        code: "WLI",
        name: "Welling",
    },
    Station {
        code: "WLM",
        name: "Williamwood",
    },
    Station {
        code: "WLN",
        name: "Wellington (Shropshire)",
    },
    Station {
        code: "WLO",
        name: "Waterloo (Merseyside)",
    },
    Station {
        code: "WLP",
        name: "Welshpool",
    },
    Station {
        code: "WLQ",
        name: "WELLINGTON (SOMERSET)",
    },
    Station {
        code: "WLS",
        name: "Woolston",
    },
    Station {
        code: "WLT",
        name: "Wallington",
    },
    Station {
        code: "WLV",
        name: "Wallasey Village",
    },
    Station {
        code: "WLW",
        name: "Welwyn North",
    },
    Station {
        code: "WLY",
        name: "Woodley",
    },
    Station {
        code: "WMA",
        name: "West Malling",
    },
    Station {
        code: "WMB",
        name: "Wembley Central",
    },
    Station {
        code: "WMC",
        name: "Wilmcote",
    },
    Station {
        code: "WMD",
        name: "Wymondham",
    },
    Station {
        code: "WME",
        name: "Woodmansterne",
    },
    Station {
        code: "WMG",
        name: "Welham Green",
    },
    Station {
        code: "WMI",
        name: "Wildmill",
    },
    Station {
        code: "WML",
        name: "Wilmslow",
    },
    Station {
        code: "WMM",
        name: "Withington (Metrolink)",
    },
    Station {
        code: "WMN",
        name: "Warminster",
    },
    Station {
        code: "WMR",
        name: "Widney Manor",
    },
    Station {
        code: "WMS",
        name: "Wemyss Bay",
    },
    Station {
        code: "WMT",
        name: "Weaste (Metrolink)",
    },
    Station {
        code: "WMW",
        name: "Walthamstow Queens Road",
    },
    Station {
        code: "WNC",
        name: "Windsor & Eton Central",
    },
    Station {
        code: "WND",
        name: "Wendover",
    },
    Station {
        code: "WNE",
        name: "Wilnecote (Staffs)",
    },
    Station {
        code: "WNF",
        name: "Winchfield",
    },
    Station {
        code: "WNG",
        name: "Waun-Gron Park",
    },
    Station {
        code: "WNH",
        name: "Warnham",
    },
    Station {
        code: "WNI",
        name: "Winchelsea New I",
    },
    Station {
        code: "WNL",
        name: "Whinhill",
    },
    Station {
        code: "WNM",
        name: "Weston Milton",
    },
    Station {
        code: "WNN",
        name: "Wennington",
    },
    Station {
        code: "WNP",
        name: "Wanstead Park",
    },
    Station {
        code: "WNR",
        name: "Windsor & Eton Riverside",
    },
    Station {
        code: "WNS",
        name: "Winnersh",
    },
    Station {
        code: "WNT",
        name: "Wandsworth Town",
    },
    Station {
        code: "WNW",
        name: "West Norwood",
    },
    Station {
        code: "WNY",
        name: "White Notley",
    },
    Station {
        code: "WOB",
        name: "Woburn Sands",
    },
    Station {
        code: "WOF",
        name: "Worcester Foregate Street",
    },
    Station {
        code: "WOH",
        name: "Woldingham",
    },
    Station {
        code: "WOK",
        name: "Woking",
    },
    Station {
        code: "WOL",
        name: "Wolverton",
    },
    Station {
        code: "WOM",
        name: "Wombwell",
    },
    Station {
        code: "WON",
        name: "Walton-on-the-Naze",
    },
    Station {
        code: "WOO",
        name: "Wool",
    },
    Station {
        code: "WOP",
        name: "Worcestershire Parkway",
    },
    Station {
        code: "WOR",
        name: "Worle",
    },
    Station {
        code: "WOS",
        name: "Worcester Shrub Hill",
    },
    Station {
        code: "WPE",
        name: "Wapping",
    },
    Station {
        code: "WPK",
        name: "Wimbledon Park (Underground)",
    },
    Station {
        code: "WPL",
        name: "Worplesdon",
    },
    Station {
        code: "WPM",
        name: "Wythenshawe Park (Metrolink)",
    },
    Station {
        code: "WPT",
        name: "Westport    (CIV)",
    },
    Station {
        code: "WRB",
        name: "Wrabness",
    },
    Station {
        code: "WRE",
        name: "Wrenbury",
    },
    Station {
        code: "WRH",
        name: "Worthing",
    },
    Station {
        code: "WRK",
        name: "Worksop",
    },
    Station {
        code: "WRL",
        name: "Wetheral",
    },
    Station {
        code: "WRM",
        name: "Wareham (Dorset)",
    },
    Station {
        code: "WRN",
        name: "West Runton",
    },
    Station {
        code: "WRO",
        name: "WHITE ROSE (LEEDS)",
    },
    Station {
        code: "WRP",
        name: "Warwick Parkway",
    },
    Station {
        code: "WRS",
        name: "Wressle",
    },
    Station {
        code: "WRT",
        name: "Worstead",
    },
    Station {
        code: "WRU",
        name: "West Ruislip",
    },
    Station {
        code: "WRW",
        name: "Warwick",
    },
    Station {
        code: "WRX",
        name: "Wrexham General",
    },
    Station {
        code: "WRY",
        name: "Wraysbury",
    },
    Station {
        code: "WSA",
        name: "West Allerton",
    },
    Station {
        code: "WSB",
        name: "Westbury",
    },
    Station {
        code: "WSE",
        name: "Winchelsea",
    },
    Station {
        code: "WSF",
        name: "Winsford",
    },
    Station {
        code: "WSH",
        name: "Wishaw",
    },
    Station {
        code: "WSL",
        name: "Walsall",
    },
    Station {
        code: "WSM",
        name: "Weston Super Mare",
    },
    Station {
        code: "WSR",
        name: "Woodsmoor",
    },
    Station {
        code: "WST",
        name: "Wood Street",
    },
    Station {
        code: "WSU",
        name: "West Sutton",
    },
    Station {
        code: "WSW",
        name: "Wandsworth Common",
    },
    Station {
        code: "WTA",
        name: "Westerhailes",
    },
    Station {
        code: "WTB",
        name: "Whitby",
    },
    Station {
        code: "WTC",
        name: "Whitchurch (Shropshire)",
    },
    Station {
        code: "WTE",
        name: "Whitlocks End",
    },
    Station {
        code: "WTF",
        name: "Whitefield (Metrolink)",
    },
    Station {
        code: "WTG",
        name: "Watlington",
    },
    Station {
        code: "WTH",
        name: "Whitehaven",
    },
    Station {
        code: "WTI",
        name: "Winnersh Triangle",
    },
    Station {
        code: "WTL",
        name: "Whitland",
    },
    Station {
        code: "WTM",
        name: "Witham",
    },
    Station {
        code: "WTN",
        name: "Whitton (London)",
    },
    Station {
        code: "WTO",
        name: "Water Orton",
    },
    Station {
        code: "WTR",
        name: "Wateringbury",
    },
    Station {
        code: "WTS",
        name: "Whatstandwell",
    },
    Station {
        code: "WTT",
        name: "Witton (West Midlands)",
    },
    Station {
        code: "WTW",
        name: "Cowes West (Redjet)",
    },
    Station {
        code: "WTY",
        name: "Witley",
    },
    Station {
        code: "WTZ",
        name: "Whitby Bus",
    },
    Station {
        code: "WVF",
        name: "Wivelsfield",
    },
    Station {
        code: "WVH",
        name: "Wolverhampton",
    },
    Station {
        code: "WWA",
        name: "Woolwich Arsenal",
    },
    Station {
        code: "WWC",
        name: "Woolwich (Elizabeth line)",
    },
    Station {
        code: "WWD",
        name: "Woolwich Dockyard",
    },
    Station {
        code: "WWI",
        name: "West Wickham",
    },
    Station {
        code: "WWL",
        name: "Whitwell",
    },
    Station {
        code: "WWM",
        name: "Westwood (Metrolink)",
    },
    Station {
        code: "WWO",
        name: "West Worthing",
    },
    Station {
        code: "WWR",
        name: "Wandsworth Road",
    },
    Station {
        code: "WWW",
        name: "Wootton Wawen",
    },
    Station {
        code: "WXC",
        name: "Wrexham Central",
    },
    Station {
        code: "WXF",
        name: "Wexford     (CIV)",
    },
    Station {
        code: "WYB",
        name: "Weybridge",
    },
    Station {
        code: "WYE",
        name: "Wye",
    },
    Station {
        code: "WYL",
        name: "Wylde Green",
    },
    Station {
        code: "WYM",
        name: "Wylam",
    },
    Station {
        code: "WYT",
        name: "Wythall",
    },
    Station {
        code: "XAA",
        name: "Galashiels (via Berwick)",
    },
    Station {
        code: "XAE",
        name: "Abingdon (via Didcot) Bus",
    },
    Station {
        code: "XAF",
        name: "Avebury Bus",
    },
    Station {
        code: "XAG",
        name: "Ardrahan",
    },
    Station {
        code: "XAH",
        name: "Callington Bus",
    },
    Station {
        code: "XAI",
        name: "Calne Bus",
    },
    Station {
        code: "XAO",
        name: "Corsham Bus",
    },
    Station {
        code: "XAP",
        name: "Dartmouth Bus",
    },
    Station {
        code: "XAQ",
        name: "Devizes Bus",
    },
    Station {
        code: "XAS",
        name: "Fowey Bus",
    },
    Station {
        code: "XAV",
        name: "Helston Bus",
    },
    Station {
        code: "XAW",
        name: "KINGSBRIDGE BUS",
    },
    Station {
        code: "XBD",
        name: "Lyneham Bus",
    },
    Station {
        code: "XBE",
        name: "Heaton Park Bus Stop",
    },
    Station {
        code: "XBH",
        name: "Marlborough Swindon Bus",
    },
    Station {
        code: "XBR",
        name: "Midsomer Norton Bus",
    },
    Station {
        code: "XBW",
        name: "Minehead Bus",
    },
    Station {
        code: "XCD",
        name: "Craughwell",
    },
    Station {
        code: "XCF",
        name: "Cardiff Airport via Rhoose",
    },
    Station {
        code: "XCG",
        name: "Okehampton Bus",
    },
    Station {
        code: "XCL",
        name: "Perranporth Bus",
    },
    Station {
        code: "XCU",
        name: "Street Bus",
    },
    Station {
        code: "XCV",
        name: "Tavistock Bus",
    },
    Station {
        code: "XDA",
        name: "Tiverton Bus",
    },
    Station {
        code: "XDC",
        name: "Wantage Bus",
    },
    Station {
        code: "XDH",
        name: "Wells Bus",
    },
    Station {
        code: "XDI",
        name: "Royal Wootton Bassett Bus",
    },
    Station {
        code: "XDJ",
        name: "Plymouth Salt Rd",
    },
    Station {
        code: "XDK",
        name: "Swindon Bus Station",
    },
    Station {
        code: "XDN",
        name: "Chippenham Nw Rd",
    },
    Station {
        code: "XDO",
        name: "Bath Bus Station",
    },
    Station {
        code: "XDP",
        name: "Totnes Station Road",
    },
    Station {
        code: "XDR",
        name: "Par Stn Bus Stop",
    },
    Station {
        code: "XDU",
        name: "Bristol T M Stn",
    },
    Station {
        code: "XDX",
        name: "Cullompton Bus",
    },
    Station {
        code: "XDY",
        name: "Dunster Bus",
    },
    Station {
        code: "XEA",
        name: "Glastonbury Bus",
    },
    Station {
        code: "XEE",
        name: "Holsworthy Bus",
    },
    Station {
        code: "XEF",
        name: "Mevagissey Bus",
    },
    Station {
        code: "XEQ",
        name: "Radstock Bus",
    },
    Station {
        code: "XET",
        name: "NUTFIELD MEMORIAL HALL",
    },
    Station {
        code: "XFI",
        name: "East M Air/Not",
    },
    Station {
        code: "XFJ",
        name: "Eden Project",
    },
    Station {
        code: "XFP",
        name: "Blenheim Palace via Oxford Parkway",
    },
    Station {
        code: "XGR",
        name: "Gort",
    },
    Station {
        code: "XGV",
        name: "Glounthaune",
    },
    Station {
        code: "XID",
        name: "IDRIDGEHAY",
    },
    Station {
        code: "XKV",
        name: "Keighley & Worth Valley Railway",
    },
    Station {
        code: "XKW",
        name: "Blackwd Buslinc",
    },
    Station {
        code: "XLB",
        name: "Leeds Bradford Airport Flyer",
    },
    Station {
        code: "XLD",
        name: "Leeds Fest Bus",
    },
    Station {
        code: "XLN",
        name: "Cholsey & Wallingford Railway",
    },
    Station {
        code: "XLO",
        name: "Loughborough University Bus",
    },
    Station {
        code: "XME",
        name: "Midleton",
    },
    Station {
        code: "XMT",
        name: "DARTMOUTH",
    },
    Station {
        code: "XMV",
        name: "Monasterevin",
    },
    Station {
        code: "XNA",
        name: "Altrincham (Metrolink)",
    },
    Station {
        code: "XNE",
        name: "Coles Bus Necbia",
    },
    Station {
        code: "XOF",
        name: "Gosport Ferry",
    },
    Station {
        code: "XPB",
        name: "Bristol Airport",
    },
    Station {
        code: "XPX",
        name: "St Agnes Bus",
    },
    Station {
        code: "XRD",
        name: "Ryde Hoverport",
    },
    Station {
        code: "XRM",
        name: "Rochdale Station (Metrolink)",
    },
    Station {
        code: "XSB",
        name: "Sixmilebrdge",
    },
    Station {
        code: "XSC",
        name: "SALCOMBE BUS",
    },
    Station {
        code: "XSO",
        name: "Southwold Bus",
    },
    Station {
        code: "XTH",
        name: "Stansted Airport Bus",
    },
    Station {
        code: "XVE",
        name: "Oxford Westgate",
    },
    Station {
        code: "XYX",
        name: "East Midlands Designer Outlet",
    },
    Station {
        code: "YAE",
        name: "Yate",
    },
    Station {
        code: "YAL",
        name: "Yalding",
    },
    Station {
        code: "YAT",
        name: "Yatton",
    },
    Station {
        code: "YEO",
        name: "Yeoford",
    },
    Station {
        code: "YET",
        name: "Yetminster",
    },
    Station {
        code: "YMH",
        name: "Yarmouth (Isle of Wight)",
    },
    Station {
        code: "YNW",
        name: "Ynyswen",
    },
    Station {
        code: "YOK",
        name: "Yoker",
    },
    Station {
        code: "YRD",
        name: "Yardley Wood",
    },
    Station {
        code: "YRK",
        name: "York",
    },
    Station {
        code: "YRM",
        name: "Yarm",
    },
    Station {
        code: "YRT",
        name: "Yorton",
    },
    Station {
        code: "YSM",
        name: "Ystrad Mynach",
    },
    Station {
        code: "YSR",
        name: "Ystrad (Rhondda)",
    },
    Station {
        code: "YVJ",
        name: "Yeovil Junction",
    },
    Station {
        code: "YVP",
        name: "Yeovil Pen Mill",
    },
    Station {
        code: "ZAD",
        name: "Aldgate",
    },
    Station {
        code: "ZAE",
        name: "Aldgate East",
    },
    Station {
        code: "ZAT",
        name: "Acton Town",
    },
    Station {
        code: "ZBB",
        name: "Barbican",
    },
    Station {
        code: "ZBG",
        name: "Bounds Green (Underground)",
    },
    Station {
        code: "ZBM",
        name: "Boston Manor (Underground)",
    },
    Station {
        code: "ZBQ",
        name: "Barons Court (Underground)",
    },
    Station {
        code: "ZBS",
        name: "Baker Street",
    },
    Station {
        code: "ZBW",
        name: "Bromley-by-Bow (Underground)",
    },
    Station {
        code: "ZBZ",
        name: "Becontree (Underground)",
    },
    Station {
        code: "ZCK",
        name: "Cockfosters (Underground)",
    },
    Station {
        code: "ZCM",
        name: "Chesham (Underground)",
    },
    Station {
        code: "ZCO",
        name: "Croxley (Underground)",
    },
    Station {
        code: "ZCW",
        name: "Canada Water",
    },
    Station {
        code: "ZDB",
        name: "Deptford Bridge",
    },
    Station {
        code: "ZDE",
        name: "Dagenham East (Underground)",
    },
    Station {
        code: "ZED",
        name: "Edgware",
    },
    Station {
        code: "ZEH",
        name: "East Ham (Underground)",
    },
    Station {
        code: "ZEK",
        name: "Embankment",
    },
    Station {
        code: "ZEL",
        name: "Elephant & Castle (Underground)",
    },
    Station {
        code: "ZEO",
        name: "Bermondsey (Underground)",
    },
    Station {
        code: "ZET",
        name: "Earls Court",
    },
    Station {
        code: "ZFD",
        name: "Farringdon",
    },
    Station {
        code: "ZFR",
        name: "Finchley Road (Underground)",
    },
    Station {
        code: "ZHA",
        name: "Hammersmith District (Underground)",
    },
    Station {
        code: "ZHB",
        name: "High Barnet (Underground)",
    },
    Station {
        code: "ZHD",
        name: "Hillingdon (Underground)",
    },
    Station {
        code: "ZHJ",
        name: "Heathrow (Underground)",
    },
    Station {
        code: "ZHR",
        name: "Holloway Road (Underground)",
    },
    Station {
        code: "ZHS",
        name: "High Street Kensington",
    },
    Station {
        code: "ZKB",
        name: "Barking (Underground)",
    },
    Station {
        code: "ZKP",
        name: "Kilburn Park (Underground)",
    },
    Station {
        code: "ZKX",
        name: "Kings Cross (Underground)",
    },
    Station {
        code: "ZLW",
        name: "Whitechapel",
    },
    Station {
        code: "ZMG",
        name: "Moorgate",
    },
    Station {
        code: "ZMH",
        name: "Mansion House (Underground)",
    },
    Station {
        code: "ZMP",
        name: "Moor Park (Underground)",
    },
    Station {
        code: "ZMV",
        name: "Maida Vale (Underground)",
    },
    Station {
        code: "ZNA",
        name: "North Acton (Underground)",
    },
    Station {
        code: "ZND",
        name: "Northwood (Underground)",
    },
    Station {
        code: "ZNP",
        name: "Newbury Park (Underground)",
    },
    Station {
        code: "ZOA",
        name: "Oakwood (Underground)",
    },
    Station {
        code: "ZPA",
        name: "Paddington (Underground)",
    },
    Station {
        code: "ZPC",
        name: "Piccadilly Circus (Underground)",
    },
    Station {
        code: "ZPS",
        name: "Plaistow (Underground)",
    },
    Station {
        code: "ZPU",
        name: "East Putney (Underground)",
    },
    Station {
        code: "ZRE",
        name: "Redbridge (Underground)",
    },
    Station {
        code: "ZRY",
        name: "Royal Oak (Underground)",
    },
    Station {
        code: "ZSI",
        name: "South Wimbledon (Underground)",
    },
    Station {
        code: "ZSK",
        name: "South Kensington",
    },
    Station {
        code: "ZSM",
        name: "Stanmore (Underground)",
    },
    Station {
        code: "ZSO",
        name: "Southwark",
    },
    Station {
        code: "ZTG",
        name: "Tower Gateway DLR",
    },
    Station {
        code: "ZTH",
        name: "Tower Hill",
    },
    Station {
        code: "ZTL",
        name: "Turnpike Lane (Underground)",
    },
    Station {
        code: "ZTU",
        name: "Turnham Green (Underground)",
    },
    Station {
        code: "ZUM",
        name: "Upminster (Underground)",
    },
    Station {
        code: "ZWA",
        name: "Waterloo (Underground)",
    },
    Station {
        code: "ZWT",
        name: "Watford Met (Underground)",
    },
    Station {
        code: "ZWV",
        name: "Warwick Avenue (Underground)",
    },
    Station {
        code: "ZWY",
        name: "Wembley Park (Underground)",
    },
    Station {
        code: "ZWZ",
        name: "West Ham (Underground)",
    },
];

/// Looks up a UK CRS station code and returns the matching station name.
#[must_use]
pub fn lookup_crs(code: &str) -> Option<&'static str> {
    if code.is_empty() {
        return None;
    }

    let normalized = code.to_ascii_uppercase();
    let index = STATIONS
        .binary_search_by_key(&normalized.as_str(), |station| station.code)
        .ok()?;
    Some(STATIONS[index].name)
}

/// Looks up a CRS code from a station name using case-insensitive exact match
/// first, then falling back to `strsim::jaro_winkler` fuzzy matching.
///
/// Returns the three-letter CRS code or `None` if no match is found above the
/// similarity threshold (0.85).
#[must_use]
pub fn crs_from_name(name: &str) -> Option<&'static str> {
    if name.is_empty() {
        return None;
    }

    let lower = deunicode::deunicode(name).to_lowercase();

    // Exact match (case-insensitive, diacritics stripped).
    for station in STATIONS {
        if deunicode::deunicode(station.name).to_lowercase() == lower {
            return Some(station.code);
        }
    }

    // Fuzzy match — pick the best Jaro-Winkler score above threshold.
    let mut best_score = 0.0_f64;
    let mut best_code: Option<&'static str> = None;

    for station in STATIONS {
        let score =
            strsim::jaro_winkler(&lower, &deunicode::deunicode(station.name).to_lowercase());
        if score > best_score {
            best_score = score;
            best_code = Some(station.code);
        }
    }

    if best_score >= 0.85 { best_code } else { None }
}

#[cfg(test)]
mod tests {
    use super::{crs_from_name, lookup_crs};

    #[test]
    fn looks_up_known_stations() {
        assert_eq!(lookup_crs("PAD"), Some("London Paddington"));
        assert_eq!(lookup_crs("KGX"), Some("London Kings Cross"));
        assert_eq!(lookup_crs("EDB"), Some("Edinburgh"));
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(lookup_crs("pad"), Some("London Paddington"));
    }

    #[test]
    fn returns_none_for_unknown_codes() {
        assert_eq!(lookup_crs("ZZZ"), None);
    }

    #[test]
    fn returns_none_for_empty_input() {
        assert_eq!(lookup_crs(""), None);
    }

    #[test]
    fn crs_from_name_exact() {
        assert_eq!(crs_from_name("London Paddington"), Some("PAD"));
        assert_eq!(crs_from_name("Edinburgh"), Some("EDB"));
    }

    #[test]
    fn crs_from_name_case_insensitive() {
        assert_eq!(crs_from_name("london paddington"), Some("PAD"));
        assert_eq!(crs_from_name("EDINBURGH"), Some("EDB"));
    }

    #[test]
    fn crs_from_name_fuzzy() {
        // "London Kings X" should fuzzy-match "London Kings Cross"
        assert_eq!(crs_from_name("London Kings Cross Station"), Some("KGX"));
    }

    #[test]
    fn crs_from_name_no_match() {
        assert_eq!(crs_from_name("Nonexistent Station 12345"), None);
    }

    #[test]
    fn crs_from_name_empty() {
        assert_eq!(crs_from_name(""), None);
    }
}
