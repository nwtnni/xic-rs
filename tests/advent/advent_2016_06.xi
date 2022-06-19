use io
use sort
use string
use vector
use vector_map

INPUT: String = new_string_from_array("\
    eqvmfqnf\n\
    jvkezmqt\n\
    zcssqwlw\n\
    cuacncgg\n\
    ikmpzpoh\n\
    dzpzobdl\n\
    qlsnuhuq\n\
    fwqnoklz\n\
    cibgplfq\n\
    ktsqfcrv\n\
    vcknjnnx\n\
    upaiaprz\n\
    bpqmolbq\n\
    dflhnpnk\n\
    heqjflch\n\
    cmewgodc\n\
    aaorgxkn\n\
    plekphpw\n\
    fcofrbnm\n\
    bmnrygtb\n\
    rqsqsqio\n\
    rnlntwxa\n\
    cxjqqfyl\n\
    jxjnxchb\n\
    kfgutxmi\n\
    cbciszxd\n\
    irakoonu\n\
    pcgfnycg\n\
    fgeivexo\n\
    ujxdaehw\n\
    ejkvrych\n\
    nhlklbgr\n\
    etjuhgry\n\
    mkgkmykm\n\
    teuhrfto\n\
    juqfslbn\n\
    tbwxabzi\n\
    ngdnwsey\n\
    amcibkyo\n\
    xlvxwqpj\n\
    vdbzuvkh\n\
    gkagbzep\n\
    kqxzkeip\n\
    bxccztho\n\
    vqrywqlc\n\
    jbzhecjc\n\
    ozkulgxo\n\
    uiwbofuk\n\
    vfwhdnao\n\
    tycxucwd\n\
    jvhuljfs\n\
    xxhqhruc\n\
    upnndiiz\n\
    andxywil\n\
    lowofbqv\n\
    iroqzrry\n\
    nmkkqqjb\n\
    daijrfna\n\
    jmcstxlq\n\
    jdefvuaa\n\
    nkbmowfi\n\
    agotazda\n\
    kufoymrn\n\
    yijwfjyi\n\
    hyqvaouj\n\
    soueuhln\n\
    oomsbkmh\n\
    buadtssf\n\
    rvgpeeza\n\
    hjiymcmd\n\
    ebgivdap\n\
    xzieipbg\n\
    ttpudwqt\n\
    hndwuncw\n\
    wqypfkvf\n\
    jqxuaipm\n\
    fzwlgxxq\n\
    ddshbtya\n\
    ardlcgyi\n\
    soznvuyw\n\
    vyizuolp\n\
    ckfaxyvs\n\
    nbsjkibi\n\
    lsrkrdzp\n\
    oqoffwxa\n\
    bdugjlsm\n\
    rtcsylfd\n\
    fezoiliq\n\
    zwpaphcb\n\
    sdlouhyf\n\
    cfejwvls\n\
    xehddxku\n\
    edhrtdcv\n\
    ehouagvy\n\
    hoyxjfsj\n\
    quggpnpx\n\
    muqbijbe\n\
    rcnniddd\n\
    kzfeiaui\n\
    sywienef\n\
    xpxftuvq\n\
    dtbhnslt\n\
    mpcpkmfa\n\
    wysutlci\n\
    fmqomicz\n\
    mhshprxr\n\
    uxwfcftt\n\
    ehbonsrl\n\
    pjobilxx\n\
    chiebfox\n\
    lqfxgyqg\n\
    vupcjatm\n\
    wfljafhc\n\
    iygojeny\n\
    gqxmgneu\n\
    nhlwllak\n\
    xnkqpulv\n\
    awijbvef\n\
    pbcrrwqo\n\
    dobsejtb\n\
    dqdoapkc\n\
    hngrxdtx\n\
    dodsxysb\n\
    bmtyelak\n\
    cctuwwvt\n\
    rytlmoyr\n\
    fqnbuxdi\n\
    irrqladc\n\
    wnvtsneg\n\
    ugqqdmlj\n\
    nljnjiod\n\
    knidxxzh\n\
    dfymoqgt\n\
    fwgtjafh\n\
    fpdioivz\n\
    tqbewmti\n\
    mcqtbbee\n\
    pivfrpou\n\
    tdyguuos\n\
    eldmvvmi\n\
    oaiqnizz\n\
    fyqpxgwa\n\
    lzcxsazq\n\
    zhsoljwz\n\
    qnzafmjl\n\
    oopnnndl\n\
    cozehoor\n\
    bspuwzxm\n\
    ubtunnep\n\
    smdhpvxr\n\
    nsvxiwje\n\
    mmqcklsm\n\
    hhxaciaq\n\
    zzgoxhws\n\
    fvntouun\n\
    skxzmzyg\n\
    znptwuqu\n\
    aknwvojo\n\
    wftmjrsf\n\
    gahrordj\n\
    oegnykag\n\
    lvlqswph\n\
    qsowvoem\n\
    sjspasfp\n\
    ygjohzfd\n\
    jeuxigsi\n\
    lgxdtudx\n\
    qadlkrel\n\
    lpfxosdq\n\
    sgaoqkzr\n\
    rtlvuhfv\n\
    ftbbsgbn\n\
    kjxttiqu\n\
    gylikswu\n\
    lquhgmrs\n\
    hxrjagjm\n\
    epxxekgx\n\
    uwwlcbrx\n\
    feincdjp\n\
    uyxhfhsc\n\
    nojuykoh\n\
    psjuuqwu\n\
    gtlohqkz\n\
    sbzsbgrw\n\
    nbhwuxfb\n\
    phmtunrh\n\
    zmfbkvgv\n\
    mjumfpia\n\
    gkubcshe\n\
    jmavrhyd\n\
    cgffkftg\n\
    msurhdct\n\
    bvchukal\n\
    psxaluvg\n\
    tvgwjhhp\n\
    chyizcxv\n\
    dumebzkd\n\
    cjpzbkzk\n\
    ngrgseyn\n\
    xmwcmaaz\n\
    puyrbiup\n\
    xxkpznis\n\
    rguwrpua\n\
    jmolhvnn\n\
    kpeqtlan\n\
    zzgvoxlp\n\
    erbintcn\n\
    kcykvysv\n\
    ixildajc\n\
    tnvgihwe\n\
    iqwgozpj\n\
    txkgyflb\n\
    vsyzebrw\n\
    ehnbcjef\n\
    hfevkbhf\n\
    wihlqtmp\n\
    vmrmnygo\n\
    ulvsuvsn\n\
    wgxnwihd\n\
    lexgbpsv\n\
    kxqcjoeb\n\
    daodpsbb\n\
    azyqmyhv\n\
    mvzcatwb\n\
    jtvqkjrv\n\
    rtdsaqqd\n\
    xrhzmnzl\n\
    wgfiwjrh\n\
    hgrgqqxm\n\
    nwwcxoyq\n\
    qlqyhpzs\n\
    ovujfily\n\
    pzvyeryk\n\
    strswprn\n\
    nrxclypc\n\
    sfusjxzi\n\
    pclbdadw\n\
    sjhggndb\n\
    xjcutuyt\n\
    qjjjeytj\n\
    qqjrkdlb\n\
    pyzodjdh\n\
    brnmlkmi\n\
    lgipidfp\n\
    ttrfbjry\n\
    iidwekro\n\
    vnwlnyma\n\
    ylxatduo\n\
    eiokdbqr\n\
    laezjjte\n\
    kkjhfsvp\n\
    buaegtpg\n\
    vzgqletc\n\
    pkdseade\n\
    nvpyxokq\n\
    yiysgxpe\n\
    xqhtubam\n\
    lcstpvke\n\
    nnskqssg\n\
    mkrbdovg\n\
    camkeppm\n\
    iqjvotay\n\
    bodlfgkj\n\
    jiigwvzc\n\
    ixpghywy\n\
    qlzyjgue\n\
    ugyjqtzn\n\
    odeuuiir\n\
    yfhianfx\n\
    seewayqj\n\
    lstpeuja\n\
    paqqnxsr\n\
    guwkidny\n\
    susussgu\n\
    ezcayehr\n\
    tdzgvcqf\n\
    vckcnsio\n\
    obawbapm\n\
    ipebazzk\n\
    tmcpmiou\n\
    hpdlfwor\n\
    ygxlfzzr\n\
    ltyxhtbx\n\
    olzqonbx\n\
    grsxreqs\n\
    bvkjcoux\n\
    fxtuxuub\n\
    fcbxdenm\n\
    smilcfvz\n\
    ewbndkiz\n\
    httsnfqu\n\
    ghorvefw\n\
    anevvqir\n\
    sajdzwho\n\
    becdemdn\n\
    vxktmxsj\n\
    xyawkeuw\n\
    pdefbxmh\n\
    yejymgfr\n\
    mipvhnsc\n\
    tjdyqpzd\n\
    rbvqirmd\n\
    mscuflvd\n\
    draqqcda\n\
    xfegqcjg\n\
    auypywpb\n\
    gitgzstq\n\
    zveqbzgt\n\
    wxrpedre\n\
    haptyecu\n\
    tkeexmhe\n\
    ujijprbd\n\
    xjiyczwq\n\
    ehpygnrr\n\
    guvejwyt\n\
    zmtsftky\n\
    wqtklwiz\n\
    lwlessio\n\
    lrknmhzd\n\
    pkdkwevt\n\
    ncryoeth\n\
    hjsqtpxu\n\
    ivmqrwok\n\
    qozgijgu\n\
    ueujvbbe\n\
    nfxgrmsd\n\
    zeetrgdl\n\
    drfbcgxo\n\
    rjjeraeb\n\
    hshozlgv\n\
    sfgvrnez\n\
    zaoctlsa\n\
    hebtzqvy\n\
    qckvuyif\n\
    wxyszmev\n\
    ddxfwklt\n\
    jqlzpfvu\n\
    wimoefwx\n\
    kabvtrno\n\
    pbebkvkm\n\
    govfwjof\n\
    xfjkvoup\n\
    fuzxcese\n\
    zbavvmyy\n\
    mwvkrnjg\n\
    gtkyelff\n\
    bffyzhnt\n\
    vlffqryw\n\
    ofncqcqw\n\
    cnzzrjjj\n\
    txpzvykz\n\
    ukkgeavq\n\
    wdnieioq\n\
    avosnedk\n\
    ipaavrqp\n\
    eeuurfat\n\
    sfhhwqzw\n\
    vjzopzad\n\
    kdbjonqz\n\
    uaksjfuc\n\
    lumpaomf\n\
    ysebmwel\n\
    dobryhxj\n\
    oaymjqwh\n\
    qjfflojj\n\
    zqmfgwre\n\
    uimjngfs\n\
    ihwelccg\n\
    yetrodjy\n\
    aifvwtws\n\
    xiyruzqr\n\
    anuvhykm\n\
    lelbjsno\n\
    csjwqotd\n\
    pptsysey\n\
    joptcdmq\n\
    tghbxpmq\n\
    jduwbxiy\n\
    obcdlahg\n\
    dxwrzytc\n\
    axfrxlgz\n\
    gepnmvel\n\
    ztmcynch\n\
    otnicgga\n\
    bdzobaoe\n\
    vkljxwnm\n\
    qvhmitgh\n\
    yflyxbjn\n\
    qshihqki\n\
    debaxqpw\n\
    fhfcjogj\n\
    huwpnaxx\n\
    jpwnrjbc\n\
    waylsrcm\n\
    aurdpcqc\n\
    yanpouht\n\
    ybwbpcak\n\
    uzvvspnj\n\
    tftluckv\n\
    uwmditoa\n\
    wsndxybi\n\
    dotcxasi\n\
    lxgmptwn\n\
    bpdmcbgt\n\
    dpjqvvck\n\
    jmgwudli\n\
    rimvxcoa\n\
    vdlacqbl\n\
    qtzwuqny\n\
    olzuzuuq\n\
    grlyyegi\n\
    mhgtadti\n\
    yrfdffzj\n\
    wbxadryy\n\
    bhaniozq\n\
    jdishqcx\n\
    kmiatkjj\n\
    asmxdrmv\n\
    riqdknna\n\
    fsuetmeg\n\
    iikajhgb\n\
    ioswsaws\n\
    yygpvtfb\n\
    egjoltik\n\
    bypcbzpk\n\
    zaumpggx\n\
    sdizezlv\n\
    xoyallwy\n\
    gicvajdl\n\
    qzowhuxa\n\
    iyftbzns\n\
    srzjxhve\n\
    xwasqzay\n\
    qznuxpqj\n\
    mlnjztxf\n\
    rxkcymao\n\
    huvxpllx\n\
    fmnrqasq\n\
    mwwigmka\n\
    yovjkmou\n\
    kvdrltte\n\
    nymvepew\n\
    vnrjykzc\n\
    unoegpvv\n\
    trrejbob\n\
    zwsdnqnb\n\
    ljsztmgl\n\
    tiznomfv\n\
    zxtxholt\n\
    csufzpiw\n\
    jgbjpucz\n\
    mpakkeil\n\
    ixmbvvbi\n\
    ejkhcxjj\n\
    zaokljpl\n\
    oeocaxdv\n\
    ytlpsbcx\n\
    hpfserxf\n\
    nzregysc\n\
    etevckof\n\
    bcqkqdvb\n\
    xzdhhick\n\
    gystpgoo\n\
    ciiyzxxr\n\
    kwstdxnn\n\
    ztregxhx\n\
    qhvkjoqe\n\
    ugirgwax\n\
    nhukpdut\n\
    yfiibmmd\n\
    cwkayjcp\n\
    ebmlabrp\n\
    kvjhyrag\n\
    wbphpfkc\n\
    ucqvhibs\n\
    dwuavsyy\n\
    jwrdsobl\n\
    hytijctt\n\
    plcumjhv\n\
    hwexsihm\n\
    ppmfzgqt\n\
    moumyuiw\n\
    zvgbsabj\n\
    yraygmws\n\
    vopzuhor\n\
    hafhljwp\n\
    gmqpchdg\n\
    yyahpihs\n\
    xvqakyyp\n\
    deamarun\n\
    yunihcvw\n\
    gcdjqqmu\n\
    kctibuxy\n\
    gcvlcfhc\n\
    ydwoxfvg\n\
    epszfvuh\n\
    xjjvwpbz\n\
    gzpdnthj\n\
    mnkrjgwz\n\
    ldfwvvfq\n\
    tydqesvl\n\
    envwzaqv\n\
    xvwyzkpe\n\
    rmpgcjeo\n\
    pkupgxup\n\
    ekqizsjl\n\
    agvenhgu\n\
    vscaqtri\n\
    rwfjrjpg\n\
    imthkcta\n\
    sjpmwqmg\n\
    fptfgekn\n\
    ohbwdbjm\n\
    ccfrphaj\n\
    gyeaqkog\n\
    onybscve\n\
    qztmoant\n\
    abjnbrpd\n\
    zompdzuf\n\
    bamomvbw\n\
    kzmmgexu\n\
    wzoxohtn\n\
    wvgmvwdt\n\
    nlgkxmbu\n\
    vyoddxyf\n\
    phmrizhk\n\
    zhksysjf\n\
    atcfvzlx\n\
    iyabqkly\n\
    rnwidjpm\n\
    cgwddumw\n\
    fcoylnzw\n\
    lsxosfra\n\
    vbcdgfiw\n\
    aenlmdgh\n\
    fvtmormn\n\
    rllxkznc\n\
    asocydmo\n\
    zcltimlr\n\
    hrqmccpt\n\
    dfmlsvtz\n\
    ntuhkbws\n\
    oziqleds\n\
    wkzbguis\n\
    coapfihl\n\
    irzpsuql\n\
    uxaowrls\n\
    tdbefhcf\n\
    wsyusuph\n\
    lpbdrmyn\n\
    slrzkkms\n\
    wqvzwiyq\n\
    vinahrsd\n\
    thsnmqjr\n\
    kwrzmakz\n\
    ifhclifl\n\
    wkqahikb\n\
    rwnchlkr\n\
    rkhpdbbk\n\
    vqnzigbf\n\
    olzziafs\n\
    qcylpbtk\n\
    fzhtmgji\n\
    qvnyctmb\n\
    ouolgwup\n\
    xkbrykjx\n\
    apbamszk\n\
    mlrlmpoh\n\
    kdneakuk\n\
    rrhhrtfk\n\
    cbgzlbgz\n\
    mfxencal\n\
    bkctqwpe\n\
    rjdxhqof\n\
    ogcbntmp\n\
    bbftqdfk\n\
    kikdidvm\n\
    mnjgwven\n\
    yurxwsge\n\
    qlrdtzad\n\
    jalffvnu\n\
    tayfycwr\n\
    jhivnvaw\n\
    yuvffepz\n\
    mwhczdkv\n\
    xltzklis\n\
    iellkyqk\n\
    krpktxhh\n\
    rkawdywu\n\
    pqqitomj\n\
    nrhhtvtv\n\
    gwerzhwc\n\
    qlsgifir\n\
    ssvyspem\n\
    udnnmvxk\n\
    albkdbsh\n\
    obxcrucu\n\
    dnyytrcx")

main(args: int[][]) {
    messages: Vector::<String> = INPUT.split('\n')

    maximum: String = new_string()
    minimum: String = new_string()

    i: int = 0
    while i < messages.get(0).size() {

        counter: VectorMap::<Integer, Integer> = new_vector_map::<Integer, Integer>()

        j: int = 0
        while j < messages.size() {
            character: Integer = new Integer.init(messages.get(j).get(i))
            count: Integer = counter.get(character)

            if count == null {
                _ = counter.insert(character, new Integer.init(1))
            } else {
                count.value = count.value + 1
            }

            j = j + 1
        }

        weights: Vector::<WeightedInteger> = new_vector::<WeightedInteger>()

        k: int = 0
        while k < counter.size() {
            weights.push(new WeightedInteger.init(
                counter.values.get(k).value,
                counter.keys.get(k).value
            ))
            k = k + 1
        }

        bubbleSort::<WeightedInteger>(weights)

        maximum.push(weights.first().value)
        minimum.push(weights.last().value)

        i = i + 1
    }

    println(maximum.get_array())
    println(minimum.get_array())
}

final class WeightedInteger {
    weight: int
    value: int

    init(weight: int, value: int): WeightedInteger {
        this.weight = weight
        this.value = value
        return this
    }

    compare(other: WeightedInteger): int {
        order: int = other.weight - weight
        if order == 0 {
            return value - other.value
        } else {
            return order
        }
    }
}

final class Integer {
    value: int

    equals(other: Integer): bool {
        return this.value == other.value
    }

    init(value: int): Integer {
        this.value = value
        return this
    }
}
