use assert
use io
use conv
use vector
use vector_map

INPUT: int[] = "\
    gbc-frperg-pubpbyngr-znantrzrag-377[rgbnp]\n\
    nij-mywlyn-wlsiayhcw-jfumncw-alumm-mbcjjcha-422[mcjwa]\n\
    pualyuhapvuhs-ibuuf-zhslz-227[uhalp]\n\
    xlrypetn-prr-lylwjdtd-665[dzoya]\n\
    zilqwikbqdm-rmttgjmiv-mvoqvmmzqvo-278[mqvio]\n\
    rgllk-bxmefuo-sdmee-geqd-fqefuzs-274[efdgl]\n\
    ugfkmewj-yjsvw-wyy-lwuzfgdgyq-814[wygfj]\n\
    lnkfaypeha-xwogap-bejwjyejc-524[uqzms]\n\
    laffe-sorozgxe-mxgjk-jek-xkykgxin-254[kxegf]\n\
    ytu-xjhwjy-hfsid-htfynsl-qtlnxynhx-411[hyntx]\n\
    vetllbybxw-xzz-mktbgbgz-709[kblty]\n\
    ixeumktoi-kmm-giwaoyozout-176[oimkt]\n\
    frqvxphu-judgh-udpsdjlqj-udeelw-uhvhdufk-647[ntbsq]\n\
    ixccb-hjj-uhvhdufk-725[hcjub]\n\
    sehheiylu-isqludwuh-xkdj-qsgkyiyjyed-634[ydehi]\n\
    yhwooebeaz-acc-ajcejaanejc-316[acejo]\n\
    qyujihctyx-vumeyn-zchuhwcha-318[hcuya]\n\
    xtwtelcj-rclop-clmmte-nzyeltyxpye-171[eltcy]\n\
    pinovwgz-mvwwdo-yzkgjthzio-941[owzgi]\n\
    htwwtxnaj-xhfajsljw-mzsy-hzxytrjw-xjwanhj-229[jwhxa]\n\
    amlqskcp-epybc-cee-pcyaosgqgrgml-652[cegpa]\n\
    fab-eqodqf-omzpk-emxqe-560[eqfmo]\n\
    bnmrtldq-fqzcd-idkkxadzm-qdrdzqbg-365[dqzbk]\n\
    ovbunmneqbhf-wryylorna-qrirybczrag-559[rbnya]\n\
    ynukcajey-xwogap-iwjwcaiajp-966[jydme]\n\
    dkqjcbctfqwu-uecxgpigt-jwpv-fgrctvogpv-128[cgptv]\n\
    ugfkmewj-yjsvw-tmffq-vwhdgqewfl-606[zfmlc]\n\
    htqtwkzq-idj-ijxnls-723[rwmzt]\n\
    kgjgrypw-epybc-aylbw-amyrgle-amlryglkclr-184[lygra]\n\
    jxdkbqfz-yrkkv-bkdfkbbofkd-705[csxut]\n\
    ujqgywfau-uzgugdslw-sfsdqkak-684[duboh]\n\
    rwcnawjcrxwju-mhn-nwprwnnarwp-823[wnrac]\n\
    eqttqukxg-rncuvke-itcuu-ujkrrkpi-102[ukrtc]\n\
    jvuzbtly-nyhkl-ibuuf-dvyrzovw-201[uvybl]\n\
    tvsnigxmpi-fewoix-wxsveki-750[ixesv]\n\
    rtqlgevkng-ejqeqncvg-ncdqtcvqta-336[prlxq]\n\
    wfummczcyx-luvvcn-nywbhifias-864[cfimn]\n\
    irdgrxzex-vxx-nfibjyfg-763[xfgir]\n\
    buzahisl-ipvohghykvbz-qlssfilhu-klclsvwtlua-591[moyzp]\n\
    dpotvnfs-hsbef-sbnqbhjoh-fhh-nbobhfnfou-831[vbmns]\n\
    owshgfarwv-lgh-kwujwl-usfvq-ghwjslagfk-164[wgfhl]\n\
    yuxufmdk-sdmpq-bxmefuo-sdmee-dqeqmdot-222[dmequ]\n\
    clotzlnetgp-clmmte-opawzjxpye-873[elptc]\n\
    mfklstdw-usfvq-kwjnauwk-268[kwfsu]\n\
    vhglnfxk-zktwx-unggr-xgzbgxxkbgz-839[gxkzb]\n\
    yrwxefpi-tpewxmg-kveww-ywiv-xiwxmrk-932[pxhgu]\n\
    shmml-cynfgvp-tenff-qrfvta-143[fmntv]\n\
    zhdsrqlchg-sodvwlf-judvv-uhdftxlvlwlrq-855[ldvhf]\n\
    kfg-jvtivk-sleep-uvjzxe-711[evjkf]\n\
    molgbzqfib-yxphbq-obpbxoze-757[bopqx]\n\
    qfmcusbwq-qobrm-qcohwbu-fsoqeiwgwhwcb-168[qwbco]\n\
    sbejpbdujwf-gmpxfs-tupsbhf-623[bfpsj]\n\
    jsehsyafy-hdsklau-yjskk-ksdwk-242[ksyad]\n\
    rwcnawjcrxwju-ljwmh-bcxajpn-823[jwcan]\n\
    excdklvo-oqq-oxqsxoobsxq-874[oqxsb]\n\
    buzahisl-jhukf-jvhapun-klwsvftlua-565[uahlf]\n\
    gpbepvxcv-snt-steadnbtci-453[tbcen]\n\
    wyvqljapsl-ihzrla-zlycpjlz-149[lzajp]\n\
    amlqskcp-epybc-cee-kylyeckclr-938[cekly]\n\
    jchipqat-qphzti-advxhixrh-895[hiapq]\n\
    tinnm-qvcqczohs-qighcasf-gsfjwqs-818[jfuek]\n\
    qyujihctyx-mwupyhayl-bohn-wihnuchgyhn-890[hynuc]\n\
    wlqqp-nvrgfezqvu-irsszk-ivjvrity-607[viqrs]\n\
    molgbzqfib-avb-cfkxkzfkd-315[bfkza]\n\
    luxciuwncpy-wuhxs-womnigyl-mylpcwy-266[ylhtr]\n\
    ugdgjxmd-bwddqtwsf-ugflsafewfl-762[qdtes]\n\
    fmsledevhsyw-nippcfier-eguymwmxmsr-438[vmsip]\n\
    xekdwvwnzkqo-xwogap-ajcejaanejc-706[aejwc]\n\
    wfummczcyx-yaa-fiacmncwm-136[bxsio]\n\
    rdadguja-tvv-ldgzhwde-375[dagve]\n\
    wsvsdkbi-qbkno-oqq-domrxyvyqi-718[qobdi]\n\
    oaxadrgx-qss-oazfmuzyqzf-300[mfedb]\n\
    hqfxxnknji-uqfxynh-lwfxx-xfqjx-125[zkwtx]\n\
    gpbepvxcv-qphzti-rdcipxcbtci-947[cpibt]\n\
    etyyx-bzmcx-bnzshmf-qdrdzqbg-443[btyez]\n\
    htqtwkzq-gzssd-qfgtwfytwd-541[ogntm]\n\
    uiovmbqk-kpwkwtibm-mvoqvmmzqvo-798[awevt]\n\
    zotts-vumeyn-xypyfijgyhn-448[qasni]\n\
    zovldbkfz-pzxsbkdbo-erkq-xznrfpfqflk-367[eunpo]\n\
    htwwtxnaj-gntmfefwitzx-hfsid-htfynsl-zxjw-yjxynsl-255[tfnwx]\n\
    vhglnfxk-zktwx-vahvhetmx-labiibgz-839[hvxab]\n\
    htqtwkzq-idj-wjhjnansl-983[rmtzn]\n\
    irgyyolokj-vrgyzoi-mxgyy-aykx-zkyzotm-358[yogkz]\n\
    ktfitzbgz-lvtoxgzxk-angm-wxitkmfxgm-943[vxmua]\n\
    uwtojhynqj-hmthtqfyj-jslnsjjwnsl-879[jhnst]\n\
    mrxivrexmsrep-gerhc-gsexmrk-hiwmkr-100[yzpuo]\n\
    hdgdovmt-bmvyz-ezggtwzvi-adivixdib-707[divgz]\n\
    lqwhuqdwlrqdo-iorzhu-ghvljq-959[qhldo]\n\
    vhkkhlbox-wrx-inkvatlbgz-397[kbhlv]\n\
    tyepcyletzylw-awldetn-rcldd-dezclrp-795[ldect]\n\
    sedikcuh-whqtu-uww-tufqhjcudj-946[uhwcd]\n\
    lsyrkjkbnyec-zvkcdsm-qbkcc-myxdksxwoxd-848[kcdsx]\n\
    fnjyxwrinm-lqxlxujcn-mnyjacvnwc-355[ncjxl]\n\
    gpbepvxcv-tvv-rdcipxcbtci-141[cvpbi]\n\
    xgsvgmotm-hgyqkz-ykxboiky-124[gkymo]\n\
    udskkaxawv-usfvq-esjcwlafy-814[uidxk]\n\
    fydelmwp-nlyoj-opalcexpye-899[elpyo]\n\
    aczupnetwp-qwzhpc-afcnsldtyr-717[cpant]\n\
    bknsykmdsfo-nio-yzobkdsyxc-926[kosyb]\n\
    xjmmjndqz-xcjxjgvoz-mzvxlpdndodji-343[fqvmn]\n\
    amjmpdsj-qaytclecp-fslr-bcqgel-782[claej]\n\
    fnjyxwrinm-ouxfna-anjlzdrbrcrxw-719[nrxaf]\n\
    qcbgiasf-ufors-pogysh-zcuwghwqg-168[gscfh]\n\
    kmjezxodgz-wpiit-mzxzdqdib-109[aypcu]\n\
    ckgvutofkj-xghhoz-uvkxgzouty-696[ajsic]\n\
    lsyrkjkbnyec-mkxni-cdybkqo-510[kybcn]\n\
    tipfxvezt-gcrjkzt-xirjj-jvimztvj-919[pofxi]\n\
    pbybeshy-cynfgvp-tenff-svanapvat-403[afnpv]\n\
    cjpibabsepvt-cvooz-usbjojoh-155[objcp]\n\
    jvyyvzpcl-lnn-ayhpupun-929[npylu]\n\
    wsvsdkbi-qbkno-zvkcdsm-qbkcc-oxqsxoobsxq-276[sbkoq]\n\
    ugdgjxmd-usfvq-ugslafy-ugflsafewfl-918[xbmpo]\n\
    nwlddtqtpo-ojp-xlcvpetyr-639[ptdlo]\n\
    nzcczdtgp-prr-opdtry-587[wsiym]\n\
    ynssr-yehpxk-wxlbzg-111[plhnx]\n\
    xjmmjndqz-xcjxjgvoz-xjiovdihzio-967[jxioz]\n\
    enqvbnpgvir-pubpbyngr-znexrgvat-585[qtsjn]\n\
    gvcskirmg-qekrixmg-ikk-xvemrmrk-126[kmrgi]\n\
    gpbepvxcv-uadltg-ejgrwphxcv-921[gpvce]\n\
    kmjezxodgz-nxvqzibzm-cpio-adivixdib-941[izdxb]\n\
    hcd-gsqfsh-xszzmpsob-sbuwbssfwbu-428[sbfhu]\n\
    nwlddtqtpo-upwwjmply-dpcgtnpd-119[pdtwl]\n\
    mbggf-msvdly-zlycpjlz-929[aonev]\n\
    lhkhszqx-fqzcd-qzaahs-btrsnldq-rdquhbd-443[qdhsz]\n\
    luxciuwncpy-xsy-uwkocmcncih-500[cuinw]\n\
    qvbmzvibqwvit-kpwkwtibm-zmkmqdqvo-564[mqvbi]\n\
    tvsnigxmpi-jpsaiv-irkmriivmrk-568[yileu]\n\
    vxupkizork-kmm-lotgtiotm-748[xymrs]\n\
    gpewwmjmih-gerhc-hiwmkr-152[lostk]\n\
    ibghopzs-gqojsbusf-vibh-rsgwub-818[bsghi]\n\
    guahyncw-luvvcn-wihnuchgyhn-552[hncug]\n\
    iruzfrtkzmv-treup-tfrkzex-ivjvrity-373[rtivz]\n\
    dsxxw-cee-ylyjwqgq-704[eqwxy]\n\
    lhkhszqx-fqzcd-eknvdq-lzmzfdldms-911[dzlqf]\n\
    oxmeeuruqp-omzpk-oamfuzs-emxqe-248[emoup]\n\
    dyz-combod-mkxni-mykdsxq-vklybkdybi-848[dkybm]\n\
    bpvctixr-qphzti-ldgzhwde-999[abmop]\n\
    kwvacumz-ozilm-jiasmb-mvoqvmmzqvo-824[tnqvi]\n\
    njmjubsz-hsbef-kfmmzcfbo-nbobhfnfou-389[luxhg]\n\
    hwbba-fag-tgceswkukvkqp-622[kabgw]\n\
    nchhg-jiasmb-lmxizbumvb-382[bmhia]\n\
    ymszqfuo-dmnnuf-emxqe-170[syxpj]\n\
    ymszqfuo-qss-abqdmfuaze-144[qsafm]\n\
    tcfkqcevkxg-hwbba-hnqygt-vtckpkpi-440[kctbg]\n\
    zloolpfsb-gbiivybxk-rpbo-qbpqfkd-705[bopfi]\n\
    slqryzjc-pyzzgr-rpyglgle-288[uanmz]\n\
    iutyaskx-mxgjk-inuiurgzk-rumoyzoiy-696[klmzy]\n\
    dpssptjwf-cvooz-efqmpznfou-311[fopsz]\n\
    dsxxw-cee-dglylagle-756[eldgx]\n\
    nwlddtqtpo-upwwjmply-xlcvpetyr-223[pltwd]\n\
    jvuzbtly-nyhkl-lnn-lunpullypun-201[tqlba]\n\
    uiovmbqk-kivlg-bmkpvwtwog-720[kpvsu]\n\
    nchhg-xtiabqk-oziaa-zmamizkp-850[aizhk]\n\
    molgbzqfib-zixppfcfba-gbiivybxk-pqloxdb-237[igmjz]\n\
    jyfvnlupj-jhukf-jvhapun-yljlpcpun-539[dmnws]\n\
    hqtyeqsjylu-sqdto-tufqhjcudj-712[cnysz]\n\
    gsvvswmzi-gerhc-gsrxemrqirx-100[dlypm]\n\
    ktwbhtvmbox-xzz-vhgmtbgfxgm-709[bgmtx]\n\
    hjgbwuladw-uzgugdslw-vwhsjlewfl-580[wlgud]\n\
    njmjubsz-hsbef-kfmmzcfbo-efqmpznfou-181[subnv]\n\
    bnknqetk-bzmcx-zbpthrhshnm-417[bhnkm]\n\
    gspsvjyp-fyrrc-jmrergmrk-126[rgjmp]\n\
    bjfutsneji-gntmfefwitzx-kqtbjw-fhvznxnynts-307[ntfjb]\n\
    sedikcuh-whqtu-rqiauj-tuiywd-270[gipnv]\n\
    hjgbwuladw-bwddqtwsf-jwsuimakalagf-294[wadbf]\n\
    encuukhkgf-uecxgpigt-jwpv-rwtejcukpi-986[ucegk]\n\
    nzydfxpc-rclop-nlyoj-nzletyr-zapcletzyd-847[lyzcn]\n\
    eqpuwogt-itcfg-lgnnadgcp-tgceswkukvkqp-518[gckpt]\n\
    nzwzcqfw-mldvpe-afcnsldtyr-171[cdfln]\n\
    ide-htrgti-snt-advxhixrh-401[hitdr]\n\
    fmsledevhsyw-gerhc-gsexmrk-erepcwmw-776[emrsw]\n\
    jvyyvzpcl-yhiipa-aljouvsvnf-201[vyaij]\n\
    chnylhuncihuf-zfiqyl-mniluay-656[hilnu]\n\
    udskkaxawv-xdgowj-klgjsyw-346[eruiv]\n\
    pbeebfvir-sybjre-qrcnegzrag-585[erbga]\n\
    aoubshwq-qobrm-obozmgwg-948[obgmq]\n\
    jvsvymbs-ibuuf-huhsfzpz-747[subfh]\n\
    qvbmzvibqwvit-jiasmb-ikycqaqbqwv-928[qbiva]\n\
    zuv-ykixkz-kmm-jkyomt-748[kmyzi]\n\
    slqryzjc-zsllw-amlryglkclr-808[lrcsy]\n\
    enzcntvat-enoovg-ybtvfgvpf-273[vntef]\n\
    iqmbazulqp-dmnnuf-oazfmuzyqzf-664[zfmqu]\n\
    yaxsnlcrun-ouxfna-uxprbcrlb-537[nruxa]\n\
    ovbunmneqbhf-cynfgvp-tenff-ratvarrevat-351[uakpm]\n\
    qzchnzbshud-idkkxadzm-rghoohmf-885[hdzkm]\n\
    fodvvlilhg-sodvwlf-judvv-fxvwrphu-vhuylfh-101[vfhld]\n\
    qvbmzvibqwvit-kpwkwtibm-apqxxqvo-798[qvbiw]\n\
    aoubshwq-pwcvonofrcig-rms-aofyshwbu-688[oswab]\n\
    hwbba-gii-fgrnqaogpv-882[gabif]\n\
    pkl-oaynap-acc-pnwejejc-186[acpej]\n\
    ltpedcxots-qphzti-ejgrwphxcv-323[ptceh]\n\
    mybbycsfo-nio-nofovyzwoxd-250[stdkc]\n\
    bgmxkgtmbhgte-ietlmbv-zktll-inkvatlbgz-397[ptrnf]\n\
    dpotvnfs-hsbef-dipdpmbuf-qvsdibtjoh-545[dbfps]\n\
    fmsledevhsyw-veqtekmrk-tpewxmg-kveww-hitevxqirx-568[evwkm]\n\
    ykjoqian-cnwza-ywjzu-ykwpejc-wymqeoepekj-628[hmfzu]\n\
    wihmogyl-aluxy-vumeyn-jolwbumcha-240[lmuya]\n\
    yuxufmdk-sdmpq-eomhqzsqd-tgzf-emxqe-664[mqdef]\n\
    wifilzof-vumeyn-guhuaygyhn-864[uyfgh]\n\
    hplazytkpo-nlyoj-cpdplcns-457[plcno]\n\
    vhglnfxk-zktwx-utldxm-hixktmbhgl-917[tursp]\n\
    jxdkbqfz-zxkav-zlxqfkd-pxibp-133[xkzbd]\n\
    mfklstdw-xdgowj-jwuwanafy-554[wadfj]\n\
    eqttqukxg-tcddkv-vtckpkpi-596[ampxv]\n\
    tpspahyf-nyhkl-jovjvshal-mpuhujpun-591[fkeyj]\n\
    vqr-ugetgv-ecpfa-eqcvkpi-ujkrrkpi-414[ekprv]\n\
    mvkccspson-bkllsd-nozvyiwoxd-952[oscdk]\n\
    ugjjgkanw-ugfkmewj-yjsvw-bwddqtwsf-kwjnauwk-528[nkliy]\n\
    wkqxodsm-lexxi-myxdksxwoxd-848[xdkmo]\n\
    tfiifjzmv-tyftfcrkv-jyzggzex-841[unmyd]\n\
    wdjcvuvmyjpn-rzvkjiduzy-mvwwdo-adivixdib-421[dvijw]\n\
    xzwrmkbqtm-jcvvg-amzdqkma-226[uonyt]\n\
    tvsnigxmpi-mrxivrexmsrep-veffmx-xvemrmrk-308[mrxev]\n\
    iehepwnu-cnwza-nwxxep-owhao-420[wenah]\n\
    fubrjhqlf-edvnhw-wudlqlqj-725[lqdfh]\n\
    wfummczcyx-wuhxs-mniluay-370[cbijt]\n\
    jchipqat-eaphixr-vgphh-sthxvc-895[hpaci]\n\
    pelbtravp-ohaal-hfre-grfgvat-169[arefg]\n\
    jshzzpmplk-jhukf-aljouvsvnf-279[jfhkl]\n\
    hwbba-ejqeqncvg-ocpcigogpv-128[cgbeo]\n\
    fnjyxwrinm-ajkkrc-cajrwrwp-745[rjwac]\n\
    mhi-lxvkxm-utldxm-tgterlbl-267[lmtxb]\n\
    jxdkbqfz-bdd-pqloxdb-237[dbqxf]\n\
    qfkkj-nzydfxpc-rclop-clmmte-xlylrpxpye-197[vzyuc]\n\
    bxaxipgn-vgpst-qxdwpopgsdjh-ytaanqtpc-hwxeexcv-687[csdop]\n\
    rdggdhxkt-eaphixr-vgphh-itrwcdadvn-245[dhgra]\n\
    qlm-pbzobq-zxkav-zlxqfkd-obzbfsfkd-471[bzfkq]\n\
    ajyqqgdgcb-qaytclecp-fslr-sqcp-rcqrgle-106[cqglr]\n\
    zgmfyxypbmsq-aylbw-amyrgle-qfgnngle-704[jmbna]\n\
    pkl-oaynap-acc-zalhkuiajp-654[apckl]\n\
    bqxnfdmhb-okzrshb-fqzrr-btrsnldq-rdquhbd-599[nszgr]\n\
    mybbycsfo-mkxni-mykdsxq-cobfsmoc-302[mbcos]\n\
    ujoon-eaphixr-vgphh-ldgzhwde-141[hdego]\n\
    iuxxuyobk-hatte-lotgtiotm-852[toiux]\n\
    muqfedyput-rkddo-huqsgkyiyjyed-608[dyuek]\n\
    mrxivrexmsrep-ikk-irkmriivmrk-230[rikme]\n\
    htqtwkzq-wfggny-ywfnsnsl-749[nwfgq]\n\
    sno-rdbqds-idkkxadzm-trdq-sdrshmf-599[dsrkm]\n\
    apuut-wpiit-gvwjmvojmt-369[tijmp]\n\
    molgbzqfib-yxphbq-tlohpelm-133[blhmo]\n\
    ugdgjxmd-wyy-hmjuzskafy-866[ydgjm]\n\
    slqryzjc-hcjjwzcyl-dglylagle-860[lcjyg]\n\
    ktwbhtvmbox-wrx-etuhktmhkr-241[psbxd]\n\
    oaddaeuhq-otaoaxmfq-ruzmzouzs-950[aouzd]\n\
    ugfkmewj-yjsvw-tmffq-klgjsyw-528[sqogh]\n\
    vrurcjah-pajmn-ajvyjprwp-ljwmh-anlnrerwp-433[jkstx]\n\
    fab-eqodqf-vqxxknqmz-ymdwqfuzs-586[qfdmx]\n\
    tpspahyf-nyhkl-kfl-klwhyatlua-123[lahky]\n\
    zntargvp-enoovg-znexrgvat-195[gnvae]\n\
    dkqjcbctfqwu-uecxgpigt-jwpv-ugtxkegu-934[gucte]\n\
    owshgfarwv-xdgowj-jwkwsjuz-320[wjgos]\n\
    gifavtkzcv-treup-jyzggzex-659[gzetv]\n\
    bjfutsneji-gfxpjy-tujwfyntsx-203[jftns]\n\
    pxtihgbsxw-ietlmbv-zktll-ybgtgvbgz-371[bgtli]\n\
    crwwv-zxkav-pefmmfkd-367[fkmvw]\n\
    sbqiiyvyut-fbqijys-whqii-tulubefcudj-998[xytos]\n\
    gvcskirmg-gsvvswmzi-fewoix-pskmwxmgw-230[gmswi]\n\
    eqttqukxg-ejqeqncvg-octmgvkpi-232[qegtc]\n\
    lqwhuqdwlrqdo-fdqgb-zrunvkrs-439[qdrlu]\n\
    tinnm-gqojsbusf-vibh-gozsg-480[gsbin]\n\
    lujbbrornm-ljwmh-orwjwlrwp-849[rwjlb]\n\
    jef-iushuj-rkddo-efuhqjyedi-868[dejuf]\n\
    szfyrqriuflj-treup-jrcvj-971[slkjz]\n\
    ltpedcxots-rpcsn-itrwcdadvn-921[cdtnp]\n\
    ohmnuvfy-xsy-jolwbumcha-968[hmouy]\n\
    gntmfefwitzx-wfintfhynaj-uqfxynh-lwfxx-wjhjnansl-905[fnwxh]\n\
    xcitgcpixdcpa-tvv-hwxeexcv-271[cxvei]\n\
    jyfvnlupj-qlssfilhu-ayhpupun-227[zbydk]\n\
    wdjcvuvmyjpn-zbb-vxlpdndodji-291[djvbn]\n\
    wfummczcyx-dyffsvyuh-xymcah-630[ycfmh]\n\
    cebwrpgvyr-wryylorna-phfgbzre-freivpr-897[opgba]\n\
    cjpibabsepvt-tdbwfohfs-ivou-fohjoffsjoh-363[fobhj]\n\
    zekvierkzferc-jtrmvexvi-ylek-wzeretzex-425[erzkv]\n\
    sgmtkzoi-pkrrehkgt-ygrky-228[kgrty]\n\
    iruzfrtkzmv-jtrmvexvi-ylek-jkfirxv-971[nvfye]\n\
    dfcxsqhwzs-pogysh-obozmgwg-870[goshw]\n\
    yuxufmdk-sdmpq-vqxxknqmz-bgdotmeuzs-326[tidcv]\n\
    iuxxuyobk-igtje-ygrky-878[mflrz]\n\
    laffe-igtje-vaxingyotm-800[aefgi]\n\
    tpspahyf-nyhkl-wshzapj-nyhzz-zhslz-643[hzpsy]\n\
    diozmivodjivg-xviyt-ozxcijgjbt-473[cmtlp]\n\
    pyknyegle-njyqrga-epyqq-pcyaosgqgrgml-314[gyqep]\n\
    bwx-amkzmb-zijjqb-tijwzibwzg-824[egorq]\n\
    drxevkzt-vxx-uvjzxe-581[xvezd]\n\
    ktfitzbgz-cxeeruxtg-wxitkmfxgm-761[txgef]\n\
    htsxzrjw-lwfij-wfggny-ywfnsnsl-801[cjidb]\n\
    oxmeeuruqp-nmewqf-pqeusz-742[eqump]\n\
    hqfxxnknji-kzeed-ojqqdgjfs-jslnsjjwnsl-671[jnsqd]\n\
    pbybeshy-rtt-phfgbzre-freivpr-221[rbepf]\n\
    pdjqhwlf-edvnhw-xvhu-whvwlqj-231[hwvdj]\n\
    gcfcnuls-aluxy-vumeyn-lywycpcha-188[cylua]\n\
    plolwdub-judgh-edvnhw-pdqdjhphqw-699[dhpwj]\n\
    udpsdjlqj-hjj-pdqdjhphqw-751[jdhpq]\n\
    amjmpdsj-pyzzgr-qyjcq-886[jmpqy]\n\
    lahxpnwrl-ljwmh-bqryyrwp-667[tifxe]\n\
    drxevkzt-avccpsvre-uvgcfpdvek-191[vcedk]\n\
    xzwrmkbqtm-rmttgjmiv-apqxxqvo-928[lmkgz]\n\
    eqnqthwn-tcddkv-fgrctvogpv-648[tvcdg]\n\
    bjfutsneji-ojqqdgjfs-ijajqturjsy-515[jqsfi]\n\
    sebehvkb-hqrryj-ixyffydw-166[siyrz]\n\
    zlkprjbo-doxab-mixpqfz-doxpp-jxohbqfkd-783[yjhzq]\n\
    eza-dpncpe-nsznzwlep-opdtry-821[sdeti]\n\
    tbxmlkfwba-yxphbq-xkxivpfp-523[slfmk]\n\
    ucynmlgxcb-hcjjwzcyl-umpiqfmn-548[cmjln]\n\
    lxwbdvna-pajmn-bljenwpna-qdwc-dbna-cnbcrwp-199[nabwc]\n\
    eadalsjq-yjsvw-wyy-jwsuimakalagf-892[ajswy]\n\
    fruurvlyh-mhoobehdq-zrunvkrs-907[rhuov]\n\
    sbqiiyvyut-vbemuh-sedjqydcudj-686[ltadr]\n\
    fkqbokxqflkxi-avb-lmboxqflkp-991[kbflq]\n\
    lhkhszqx-fqzcd-atmmx-kzanqzsnqx-677[qzxah]\n\
    cebwrpgvyr-hafgnoyr-cynfgvp-tenff-qrcnegzrag-793[rxmql]\n\
    ajmrxjlcren-cxy-bnlanc-ouxfna-lxwcjrwvnwc-927[cnxaj]\n\
    hqtyeqsjylu-isqludwuh-xkdj-tulubefcudj-244[udjlq]\n\
    vdzonmhydc-azrjds-cdudknoldms-157[lzowh]\n\
    uwtojhynqj-kqtbjw-zxjw-yjxynsl-333[grmkp]\n\
    myxcewob-qbkno-oqq-ecob-docdsxq-614[oqbcd]\n\
    rkpqxyib-gbiivybxk-abpfdk-419[bikpx]\n\
    zlilocri-zelzlixqb-qbzeklildv-497[ucyzj]\n\
    pinovwgz-xviyt-vivgtndn-499[vingt]\n\
    gcfcnuls-aluxy-luvvcn-xymcah-318[cluan]\n\
    sebehvkb-fbqijys-whqii-husuylydw-400[bhisy]\n\
    rdchjbtg-vgpst-eaphixr-vgphh-rjhidbtg-htgkxrt-323[hgtrp]\n\
    pualyuhapvuhs-jshzzpmplk-wshzapj-nyhzz-zlycpjlz-175[zphla]\n\
    atyzghrk-sgmtkzoi-hgyqkz-yzuxgmk-228[gkzyh]\n\
    ohmnuvfy-mwupyhayl-bohn-yhachyylcha-630[hyacl]\n\
    oxjmxdfkd-avb-pqloxdb-211[dxboa]\n\
    iqmbazulqp-bdavqofuxq-omzpk-ruzmzouzs-482[zqumo]\n\
    zsxyfgqj-gzssd-wjxjfwhm-619[jsfgw]\n\
    qvbmzvibqwvit-jcvvg-abwziom-512[tcrkb]\n\
    xgjougizobk-lruckx-aykx-zkyzotm-826[koxzg]\n\
    bkzrrhehdc-rbzudmfdq-gtms-zmzkxrhr-755[rzdhm]\n\
    myxcewob-qbkno-cmkfoxqob-rexd-zebmrkcsxq-302[syrvm]\n\
    zekvierkzferc-treup-tfrkzex-uvgrikdvek-867[ekrvz]\n\
    cvabijtm-lgm-bziqvqvo-330[vbimq]\n\
    vhglnfxk-zktwx-ktuubm-vhgmtbgfxgm-553[gkmtx]\n\
    xst-wigvix-fyrrc-vieguymwmxmsr-490[yentm]\n\
    ktfitzbgz-fbebmtkr-zktwx-xzz-lmhktzx-111[ztkbx]\n\
    vdzonmhydc-eknvdq-sqzhmhmf-963[xacdu]\n\
    dmpuamofuhq-otaoaxmfq-efadmsq-742[amfoq]\n\
    htqtwkzq-gfxpjy-wjxjfwhm-827[fnred]\n\
    sbnqbhjoh-xfbqpojafe-cbtlfu-dpoubjonfou-311[kezry]\n\
    qyujihctyx-vumeyn-lywycpcha-604[ychua]\n\
    ide-htrgti-tvv-uxcpcrxcv-973[ctvir]\n\
    bxaxipgn-vgpst-snt-gtprfjxhxixdc-791[xgpti]\n\
    nbhofujd-dipdpmbuf-dvtupnfs-tfswjdf-363[dfpub]\n\
    apuut-nxvqzibzm-cpio-mznzvmxc-291[zmcin]\n\
    uzfqdzmfuazmx-otaoaxmfq-pqhqxabyqzf-768[pzmry]\n\
    tpspahyf-nyhkl-ibuuf-klwhyatlua-253[xkrsz]\n\
    iqmbazulqp-vqxxknqmz-efadmsq-950[jrnox]\n\
    bpvctixr-rpcsn-igpxcxcv-375[cpxir]\n\
    ytu-xjhwjy-uqfxynh-lwfxx-yjhmstqtld-489[xyhjt]\n\
    qvbmzvibqwvit-ntwemz-kwvbiqvumvb-720[vbimq]\n\
    mhi-lxvkxm-ietlmbv-zktll-vnlmhfxk-lxkobvx-553[eusnm]\n\
    tpspahyf-nyhkl-jhukf-wbyjohzpun-487[hpyfj]\n\
    avw-zljyla-qlssfilhu-zlycpjlz-929[lzajs]\n\
    sawlkjevaz-xwogap-skngodkl-290[akglo]\n\
    xgjougizobk-laffe-lruckx-gtgreyoy-774[goefk]\n\
    aoubshwq-qobrm-qcohwbu-qighcasf-gsfjwqs-948[qsbho]\n\
    wifilzof-vumeyn-lyuwkocmcncih-968[avixc]\n\
    uiovmbqk-xtiabqk-oziaa-mvoqvmmzqvo-382[moqva]\n\
    sawlkjevaz-oywrajcan-dqjp-lqnydwoejc-342[ajwcd]\n\
    kfg-jvtivk-jtrmvexvi-ylek-rercpjzj-529[jvekr]\n\
    houngfgxjuay-igtje-giwaoyozout-228[gouai]\n\
    gcfcnuls-aluxy-mwupyhayl-bohn-ijyluncihm-916[tacdb]\n\
    cjpibabsepvt-cvooz-ufdiopmphz-155[pobci]\n\
    iuxxuyobk-igtje-sgtgmksktz-878[uwcvx]\n\
    thnulapj-ibuuf-ylzlhyjo-305[sfdnr]\n\
    xzwrmkbqtm-rmttgjmiv-zmamizkp-434[ifpry]\n\
    yhwooebeaz-zua-yqopkian-oanreya-680[aoeyn]\n\
    tfcfiwlc-wcfnvi-wzeretzex-243[cefwi]\n\
    guahyncw-xsy-uwkocmcncih-864[qsmtb]\n\
    ovbunmneqbhf-rtt-qrfvta-689[zymsd]\n\
    rgllk-eomhqzsqd-tgzf-ymdwqfuzs-638[qzdfg]\n\
    ryexqpqhteki-sqdto-seqjydw-skijecuh-iuhlysu-946[eqshi]\n\
    avw-zljyla-qlssfilhu-mpuhujpun-383[luahj]\n\
    pynffvsvrq-onfxrg-znantrzrag-143[nrfag]\n\
    ikhcxvmbex-xzz-phkdlahi-839[cstrx]\n\
    mvhkvbdib-wvnfzo-zibdizzmdib-187[bizdv]\n\
    ipvohghykvbz-wshzapj-nyhzz-huhsfzpz-747[hzpsv]\n\
    htqtwkzq-hmthtqfyj-ijuqtdrjsy-151[tqhjy]\n\
    xzz-ftgtzxfxgm-865[tupfq]\n\
    jyfvnlupj-jhukf-klwhyatlua-747[jydsc]\n\
    mbiyqoxsm-mkxni-kxkvicsc-510[ikmxc]\n\
    bgmxkgtmbhgte-ietlmbv-zktll-labiibgz-163[bglti]\n\
    vdzonmhydc-bqxnfdmhb-rbzudmfdq-gtms-lzmzfdldms-469[arkps]\n\
    forwcoqhwjs-gqojsbusf-vibh-rsgwub-688[dgqsb]\n\
    qcffcgwjs-pogysh-qcbhowbasbh-688[bchsf]\n\
    apuut-xviyt-yzqzgjkhzio-317[zituy]\n\
    ide-htrgti-qjccn-jhtg-ithixcv-479[itchg]\n\
    kgjgrypw-epybc-hcjjwzcyl-cleglccpgle-262[qphrv]\n\
    atyzghrk-lruckx-jkvruesktz-384[krtuz]\n\
    hqtyeqsjylu-rqiauj-vydqdsydw-998[gqeba]\n\
    uwtojhynqj-gfxpjy-qfgtwfytwd-177[fjtwy]\n\
    nglmtuex-xzz-ftgtzxfxgm-839[xgtzf]\n\
    ncjzrpytn-clmmte-epnsyzwzrj-951[yqksh]\n\
    gntmfefwitzx-gzssd-htsyfnsrjsy-333[cngmk]\n\
    qcbgiasf-ufors-qvcqczohs-hfowbwbu-168[bcfoq]\n\
    wlqqp-gcrjkzt-xirjj-dribvkzex-529[ycrxs]\n\
    drxevkzt-irdgrxzex-jtrmvexvi-ylek-glityrjzex-321[erxit]\n\
    ovbunmneqbhf-qlr-znexrgvat-559[nbeqr]\n\
    bwx-amkzmb-jiasmb-camz-bmabqvo-512[bmazc]\n\
    vcibutulxiom-vohhs-womnigyl-mylpcwy-838[fczlm]\n\
    fmsledevhsyw-ikk-hitpscqirx-230[owjnv]\n\
    ykhknbqh-ywjzu-ykwpejc-odellejc-940[xguqm]\n\
    nsyjwsfyntsfq-gzssd-uzwhmfxnsl-203[sfnwy]\n\
    mtzslklcozfd-clmmte-nzyeltyxpye-301[tmlui]\n\
    dsxxw-cee-kypicrgle-106[ecxdg]\n\
    ujqgywfau-aflwjfslagfsd-tskcwl-ghwjslagfk-476[fagls]\n\
    nchhg-jcvvg-mvoqvmmzqvo-642[vmcgh]\n\
    cjpibabsepvt-tdbwfohfs-ivou-efqmpznfou-831[mvwiq]\n\
    votubcmf-njmjubsz-hsbef-dboez-dpbujoh-fohjoffsjoh-129[izchs]\n\
    njmjubsz-hsbef-fhh-nbobhfnfou-337[unims]\n\
    iwcjapey-lhwopey-cnwoo-hkceopeyo-576[oecpw]\n\
    ydjuhdqjyedqb-fbqijys-whqii-efuhqjyedi-322[qdijy]\n\
    bknsykmdsfo-lkcuod-mecdywob-cobfsmo-250[obcdk]\n\
    sbqiiyvyut-zubboruqd-cqdqwucudj-530[uqbdc]\n\
    etaqigpke-dcumgv-vgejpqnqia-960[egqai]\n\
    ykjoqian-cnwza-nwxxep-paydjkhkcu-134[pcdmt]\n\
    iehepwnu-cnwza-lhwopey-cnwoo-nawymqeoepekj-108[ewnop]\n\
    vagreangvbany-rtt-phfgbzre-freivpr-221[raegv]\n\
    surmhfwloh-sodvwlf-judvv-xvhu-whvwlqj-595[vhwlu]\n\
    qekrixmg-ikk-gywxsqiv-wivzmgi-256[jsykh]\n\
    sno-rdbqds-bgnbnkzsd-otqbgzrhmf-495[rypqa]\n\
    guahyncw-vohhs-nywbhifias-214[hains]\n\
    sno-rdbqds-atmmx-bnmszhmldms-365[posvl]\n\
    zovldbkfz-zxkav-zlxqfkd-zlkqxfkjbkq-575[zrqmk]\n\
    ykhknbqh-zua-owhao-888[hakob]\n\
    xmrrq-vqw-ugflsafewfl-372[isvjx]\n\
    wdjcvuvmyjpn-wpiit-vxlpdndodji-395[cvdlm]\n\
    wyvqljapsl-ihzrla-zopwwpun-123[lpwaz]\n\
    kdijqrbu-tou-husuylydw-816[uvcwx]\n\
    fhezusjybu-fbqijys-whqii-husuylydw-764[uyhis]\n\
    jyfvnlupj-kfl-mpuhujpun-773[ujpfl]\n\
    hafgnoyr-pubpbyngr-nanylfvf-715[nkyzs]\n\
    jfifqxov-doxab-oxyyfq-absbilmjbkq-341[qmgrk]\n\
    nij-mywlyn-wuhxs-wiuncha-uwkocmcncih-188[cnwhi]\n\
    amjmpdsj-afmamjyrc-ylyjwqgq-470[jmayq]\n\
    rdggdhxkt-eaphixr-vgphh-jhtg-ithixcv-921[yvuxl]\n\
    ucynmlgxcb-qaytclecp-fslr-dglylagle-184[tudeg]\n\
    dpmpsgvm-tdbwfohfs-ivou-sfdfjwjoh-363[qhgxy]\n\
    bqvvu-ykhknbqh-fahhuxawj-wymqeoepekj-498[hekqa]\n\
    qczcftiz-xszzmpsob-sbuwbssfwbu-818[sbzcf]\n\
    aietsrmdih-hci-wlmttmrk-360[imthr]\n\
    xst-wigvix-ikk-qevoixmrk-256[ikxve]\n\
    nzydfxpc-rclop-nzwzcqfw-mfyyj-opalcexpye-405[cpyfz]\n\
    frqvxphu-judgh-udeelw-uhdftxlvlwlrq-933[ludhe]\n\
    jsehsyafy-jsttal-hmjuzskafy-892[sajyf]\n\
    zbytomdsvo-mrymyvkdo-vyqscdsmc-276[mydos]\n\
    tcorcikpi-ecpfa-eqcvkpi-fgxgnqrogpv-934[jziot]\n\
    ytu-xjhwjy-hfsid-wjxjfwhm-905[jhwfx]\n\
    hjgbwuladw-tmffq-suimakalagf-554[afglm]\n\
    pyknyegle-zsllw-nspafyqgle-730[leygn]\n\
    gifavtkzcv-avccpsvre-uvjzxe-607[zhayg]\n\
    bpvctixr-snt-tcvxcttgxcv-973[vrteq]\n\
    wyvqljapsl-jovjvshal-zlycpjlz-175[nrwfe]\n\
    kwzzwaqdm-lgm-aitma-122[amwzd]\n\
    iqmbazulqp-dmnnuf-qzsuzqqduzs-690[qhzsm]\n\
    udskkaxawv-xdgowj-xafsfuafy-138[nailf]\n\
    ipvohghykvbz-wshzapj-nyhzz-yljlpcpun-929[lwyvn]\n\
    forwcoqhwjs-foppwh-hsqvbczcum-636[chowf]\n\
    pualyuhapvuhs-msvdly-svnpzapjz-903[pasuv]\n\
    xgjougizobk-vrgyzoi-mxgyy-xkykgxin-436[tjykb]\n\
    sedikcuh-whqtu-rkddo-cqdqwucudj-348[brlqi]\n\
    elrkdcdugrxv-fdqgb-uhfhlylqj-465[dlfgh]\n\
    mhi-lxvkxm-lvtoxgzxk-angm-ltexl-917[xlmgk]\n\
    bqvvu-fahhuxawj-yqopkian-oanreya-212[cpdwf]\n\
    buzahisl-jhukf-thyrlapun-903[hualb]\n\
    rgllk-oaxadrgx-nmewqf-pqbxakyqzf-690[aqxfg]\n\
    iuruxlar-irgyyolokj-jek-ynovvotm-488[ohpdn]\n\
    xmtjbzidx-nxvqzibzm-cpio-yzkgjthzio-811[zixbj]\n\
    xmrrq-ugfkmewj-yjsvw-usfvq-ugslafy-ghwjslagfk-866[fgsju]\n\
    yhtwhnpun-wshzapj-nyhzz-vwlyhapvuz-851[hznpw]\n\
    zgmfyxypbmsq-zyqicr-kypicrgle-340[ycgim]\n\
    uwtojhynqj-hfsid-htfynsl-wjfhvznxnynts-489[nhfjs]\n\
    fab-eqodqf-dmnnuf-bgdotmeuzs-196[dfbem]\n\
    wifilzof-wuhxs-wiuncha-lyuwkocmcncih-578[ciwhu]\n\
    gspsvjyp-veffmx-pefsvexsvc-516[svefp]\n\
    yknnkoera-xwogap-bejwjyejc-732[ejakn]\n\
    nsyjwsfyntsfq-gfxpjy-knsfshnsl-333[sfnyj]\n\
    fodvvlilhg-gbh-ghvljq-595[vgprj]\n\
    nuatmlmdpage-bxmefuo-sdmee-emxqe-482[emadu]\n\
    jvyyvzpcl-msvdly-vwlyhapvuz-903[vylpz]\n\
    fruurvlyh-iorzhu-vdohv-517[hruvo]\n\
    houngfgxjuay-hgyqkz-ykxboiky-618[ygkho]\n\
    gsrwyqiv-kvehi-gerhc-gsexmrk-xiglrspskc-750[grsei]\n\
    pualyuhapvuhs-qlssfilhu-zhslz-799[hlsua]\n\
    nwlddtqtpo-nlyoj-nzletyr-opdtry-119[tdlno]\n\
    fydelmwp-prr-nfdezxpc-dpcgtnp-535[pdcef]\n\
    qmpmxevc-kvehi-jpsaiv-hizipstqirx-672[ipveh]\n\
    nzwzcqfw-nlyoj-nzletyr-opawzjxpye-587[znwye]\n\
    bpvctixr-rwdrdapit-hpath-973[prtad]\n\
    gzefmnxq-omzpk-oazfmuzyqzf-430[zfmoq]\n\
    wpuvcdng-hnqygt-vgejpqnqia-102[tmdxr]\n\
    aoubshwq-foppwh-igsf-hsghwbu-610[tsrzk]\n\
    wihmogyl-aluxy-mwupyhayl-bohn-mbcjjcha-422[vuypz]\n\
    cqwdujys-uww-sedjqydcudj-478[djuwc]\n\
    votubcmf-tdbwfohfs-ivou-sfbdrvjtjujpo-883[fobjt]\n\
    gpbepvxcv-ytaanqtpc-apqdgpidgn-427[pagcd]\n\
    bnknqetk-eknvdq-qdrdzqbg-885[qdknb]\n\
    uwtojhynqj-wfruflnsl-hfsid-wjxjfwhm-541[fjwhl]\n\
    zhdsrqlchg-fodvvlilhg-gbh-frqwdlqphqw-361[hlqdg]\n\
    cvabijtm-jcvvg-lmxizbumvb-174[vbmci]\n\
    fruurvlyh-fdqgb-ghsorbphqw-205[hrbfg]\n\
    pualyuhapvuhs-msvdly-thyrlapun-279[uahlp]\n\
    iehepwnu-cnwza-ydkykhwpa-ajcejaanejc-212[oqwrn]\n\
    bqvvu-xwogap-yqopkian-oanreya-680[ckqtm]\n\
    ktwbhtvmbox-vahvhetmx-mxvaghehzr-917[hvmtx]\n\
    uzfqdzmfuazmx-omzpk-oamfuzs-fqotzaxask-274[mnilo]\n\
    gntmfefwitzx-idj-rfsfljrjsy-931[fjirs]\n\
    tcrjjzwzvu-tfiifjzmv-tyftfcrkv-vexzevvizex-399[vzfte]\n\
    oaddaeuhq-vqxxknqmz-qzsuzqqduzs-404[qzdua]\n\
    sorozgxe-mxgjk-yigbktmkx-natz-zxgototm-514[hejid]\n\
    eadalsjq-yjsvw-ujqgywfau-tskcwl-kwjnauwk-554[wajks]\n\
    lxuxaodu-rwcnawjcrxwju-ljwmh-fxatbqxy-693[xwaju]\n\
    plolwdub-judgh-fdqgb-frdwlqj-vdohv-153[dlbfg]\n\
    kdijqrbu-jef-iushuj-sqdto-seqjydw-kiuh-juijydw-218[iqtvx]\n\
    tfejldvi-xiruv-tfcfiwlc-srjbvk-jyzggzex-243[fijvc]\n\
    jchipqat-ltpedcxots-uadltg-tcvxcttgxcv-609[ezynj]\n\
    ryexqpqhteki-sxesebqju-udwyduuhydw-816[eudqy]\n\
    iuxxuyobk-xgjougizobk-hatte-sgtgmksktz-436[pwdlc]\n\
    gcfcnuls-aluxy-wuhxs-wiuncha-guleyncha-136[ucahl]\n\
    ugfkmewj-yjsvw-usfvq-ugslafy-mkwj-lwklafy-476[ohqre]\n\
    laffe-vxupkizork-vrgyzoi-mxgyy-uvkxgzouty-488[awgqz]\n\
    eqttqukxg-hnqygt-rwtejcukpi-570[tqegk]\n\
    yuxufmdk-sdmpq-ngzzk-ruzmzouzs-534[zumdk]\n\
    ktwbhtvmbox-ietlmbv-zktll-kxtvjnblbmbhg-553[btlkm]\n\
    qxdwpopgsdjh-eaphixr-vgphh-prfjxhxixdc-999[hpxdg]\n\
    bnmrtldq-fqzcd-oqnidbshkd-qzaahs-cdoknxldms-703[lxvwe]\n\
    gokzyxsjon-nio-gybucryz-172[oygnz]\n\
    cqwdujys-uww-cqhaujydw-660[wucdj]\n\
    mbggf-pualyuhapvuhs-msvdly-aljouvsvnf-123[ngwhl]\n\
    crwwv-oxaflxzqfsb-zelzlixqb-ixyloxqlov-913[lxoqz]\n\
    qlm-pbzobq-ciltbo-abmxoqjbkq-861[bqolm]\n\
    oqnidbshkd-dff-rghoohmf-313[dfhob]\n\
    lzfmdshb-eknvdq-cdudknoldms-937[dklmn]\n\
    wsvsdkbi-qbkno-nio-ecob-docdsxq-614[jsetb]\n\
    zlilocri-zxkav-zlxqfkd-qoxfkfkd-835[kflxz]\n\
    wlqqp-upv-vexzevvizex-165[vepqx]\n\
    vcibutulxiom-vumeyn-womnigyl-mylpcwy-838[myilu]\n\
    pelbtravp-cynfgvp-tenff-svanapvat-663[kzmfp]\n\
    xgvnndadzy-wpiit-jkzmvodjin-421[dinjv]\n\
    foadouwbu-pogysh-fsqswjwbu-480[osuwb]\n\
    yrwxefpi-hci-wxsveki-308[iewxc]\n\
    tmrszakd-azrjds-otqbgzrhmf-105[rzadm]\n\
    sbnqbhjoh-dboez-dpbujoh-usbjojoh-155[bohjd]\n\
    eqnqthwn-gii-fgrctvogpv-908[ginqt]\n\
    uiovmbqk-jcvvg-amzdqkma-356[mvakq]\n\
    sbejpbdujwf-gmpxfs-pqfsbujpot-857[pbfjs]\n\
    ide-htrgti-ytaanqtpc-stepgibtci-531[mnyed]\n\
    aietsrmdih-glsgspexi-gywxsqiv-wivzmgi-230[igsem]\n\
    htqtwkzq-xhfajsljw-mzsy-zxjw-yjxynsl-931[cmkfr]\n\
    ckgvutofkj-xgjougizobk-yigbktmkx-natz-lotgtiotm-436[tgkoi]\n\
    nwlddtqtpo-upwwjmply-fdpc-epdetyr-509[pdtwe]\n\
    irdgrxzex-sleep-jyzggzex-373[tvnma]\n\
    crwwv-zxkav-qoxfkfkd-939[lyjmh]\n\
    ejpanjwpekjwh-nwzekwypera-oywrajcan-dqjp-nawymqeoepekj-368[zmuyt]\n\
    lzfmdshb-rbzudmfdq-gtms-knfhrshbr-495[bdfhm]\n\
    nchhg-rmttgjmiv-uizsmbqvo-252[mghit]\n\
    amjmpdsj-aylbw-rpyglgle-626[lagjm]\n\
    dfcxsqhwzs-pogysh-qighcasf-gsfjwqs-220[sfghq]\n\
    xjgjmapg-nxvqzibzm-cpio-nojmvbz-707[jmzbg]\n\
    zntargvp-enoovg-qrirybczrag-663[scjtg]\n\
    bkzrrhehdc-qzaahs-qdzbpthrhshnm-391[zjbto]\n\
    hafgnoyr-wryylorna-erprvivat-429[rayno]\n\
    apwmeclga-afmamjyrc-rpyglgle-262[aglmc]\n\
    jsvagsulanw-jsttal-hmjuzskafy-606[asjlt]\n\
    bnknqetk-lhkhszqx-fqzcd-cxd-zmzkxrhr-651[kzhqx]\n\
    ykhknbqh-nwxxep-nayaerejc-966[enahk]\n\
    vrurcjah-pajmn-kdwwh-cajrwrwp-667[rwajc]\n\
    vhehkyne-utldxm-vhgmtbgfxgm-891[ghmet]\n\
    zotts-dyffsvyuh-xymcah-812[yfhst]\n\
    vhglnfxk-zktwx-vtgwr-vnlmhfxk-lxkobvx-319[gvnom]\n\
    ajvyjprwp-mhn-nwprwnnarwp-563[npwra]\n\
    guahyncw-chnylhuncihuf-jfumncw-alumm-guleyncha-110[zjirh]\n\
    hwdtljsnh-jll-xfqjx-801[jlhxd]\n\
    xjgjmapg-mvwwdo-xjiovdihzio-525[ijodg]\n\
    pybgmyargtc-zgmfyxypbmsq-zsllw-asqrmkcp-qcptgac-262[cgmpy]\n\
    aflwjfslagfsd-hdsklau-yjskk-esfsywewfl-528[sflak]\n\
    lugjuacha-dyffsvyuh-xypyfijgyhn-708[yfhua]\n\
    lxaaxbren-mhn-cnlqwxuxph-823[nzsvm]\n\
    sehheiylu-tou-cqdqwucudj-738[xciqn]\n\
    slqryzjc-djmucp-qrmpyec-808[sznhq]\n\
    ykjoqian-cnwza-bhksan-opknwca-264[ankco]\n\
    pualyuhapvuhs-lnn-bzly-alzapun-721[auzfj]\n\
    tfiifjzmv-wcfnvi-jkfirxv-997[fivjc]\n\
    lsyrkjkbnyec-mkxni-mykdsxq-vyqscdsmc-562[ksycm]\n\
    fnjyxwrinm-lujbbrornm-ajkkrc-nwprwnnarwp-927[zmyco]\n\
    pyknyegle-amlqskcp-epybc-hcjjwzcyl-qcptgacq-860[cpyel]\n\
    rzvkjiduzy-ezggtwzvi-kpmxcvndib-811[zivdg]\n\
    wyvqljapsl-yhiipa-bzly-alzapun-773[alpyi]\n\
    joufsobujpobm-dipdpmbuf-bdrvjtjujpo-415[gvkud]\n\
    zloolpfsb-zxkav-zlxqfkd-lmboxqflkp-393[lfkox]\n\
    zilqwikbqdm-jcvvg-kwvbiqvumvb-174[vbiqk]\n\
    kzeed-wfggny-xmnuunsl-853[negud]\n\
    ftzgxmbv-xzz-phkdlahi-657[grbhi]\n\
    bnqqnrhud-bzmcx-sqzhmhmf-131[hmqbn]\n\
    zntargvp-pnaql-pbngvat-nanylfvf-169[napvf]\n\
    jxdkbqfz-pzxsbkdbo-erkq-absbilmjbkq-315[uzmcf]\n\
    jshzzpmplk-buzahisl-kfl-klzpnu-695[lzkph]\n\
    pualyuhapvuhs-msvdly-jbzavtly-zlycpjl-825[lyapu]\n\
    lujbbrornm-ouxfna-xynajcrxwb-667[bnrxa]\n\
    dmpuamofuhq-nmewqf-pqhqxabyqzf-482[lndmj]\n\
    cvabijtm-moo-zmikycqaqbqwv-148[mqabc]\n\
    wyvqljapsl-msvdly-zlycpjlz-435[lyjps]\n\
    fmsledevhsyw-ikk-gywxsqiv-wivzmgi-204[isvwe]\n\
    ide-htrgti-snt-sthxvc-297[tyvnc]\n\
    guahyncw-luvvcn-qilembij-292[tcrsd]\n\
    udskkaxawv-wyy-kwjnauwk-710[kwauy]\n\
    aczupnetwp-clmmte-dezclrp-379[ynpmz]\n\
    ikhcxvmbex-ietlmbv-zktll-vnlmhfxk-lxkobvx-449[lxkvb]\n\
    rzvkjiduzy-xcjxjgvoz-rjmfncjk-707[tmnki]\n\
    enzcntvat-cynfgvp-tenff-ynobengbel-923[neftb]\n\
    vkrhzxgbv-bgmxkgtmbhgte-lvtoxgzxk-angm-kxtvjnblbmbhg-111[iwvbg]\n\
    esyfwlau-tskcwl-jwsuimakalagf-398[ywmzb]\n\
    lhkhszqx-fqzcd-bzmcx-nodqzshnmr-287[zhqcd]\n\
    nzwzcqfw-ojp-dstaatyr-977[dsznk]\n\
    qfkkj-xlrypetn-nlyoj-xlcvpetyr-691[lczde]\n\
    wifilzof-luvvcn-nywbhifias-552[sxghy]\n\
    nchhg-kivlg-zmamizkp-928[ghikm]\n\
    tipfxvezt-tcrjjzwzvu-upv-kirzezex-295[zetvi]\n\
    gsvvswmzi-tpewxmg-kveww-gsrxemrqirx-698[wegmr]\n\
    pynffvsvrq-sybjre-ernpdhvfvgvba-663[epsqt]\n\
    sedikcuh-whqtu-vbemuh-udwyduuhydw-894[udhwe]\n\
    tmrszakd-bzmcx-rsnqzfd-183[zdmrs]\n\
    zilqwikbqdm-jcvvg-wxmzibqwva-798[iqvwb]\n\
    lejkrscv-jtrmvexvi-ylek-nfibjyfg-815[ejvfi]\n\
    zsxyfgqj-jll-qtlnxynhx-151[lxjnq]\n\
    gbc-frperg-onfxrg-qrirybczrag-923[rgbcf]\n\
    xjgjmapg-kgvnodx-bmvnn-nvgzn-343[ngvjm]\n\
    dmybmsuzs-ngzzk-mocgueufuaz-534[uzmgs]\n\
    dmpuamofuhq-omzpk-oamfuzs-pqbxakyqzf-482[mafop]\n\
    fbebmtkr-zktwx-unggr-ehzblmbvl-787[begkl]\n\
    zntargvp-enoovg-ybtvfgvpf-481[vgfno]\n\
    fubrjhqlf-gbh-vhuylfhv-933[hfblu]\n\
    fruurvlyh-fdqgb-frdwlqj-whfkqrorjb-569[tmdlw]\n\
    ixccb-udeelw-ghvljq-335[nibrq]\n\
    tcorcikpi-dwppa-fgukip-570[qnzgc]\n\
    ibghopzs-pibbm-rsdzcmasbh-428[bshim]\n\
    apuut-wpiit-nzmqdxzn-889[inptu]\n\
    qzoggwtwsr-pibbm-igsf-hsghwbu-246[gbswh]\n\
    atyzghrk-yigbktmkx-natz-uvkxgzouty-488[pxeoy]\n\
    mbiyqoxsm-mkxni-bokmaescsdsyx-796[erynw]\n\
    qxdwpopgsdjh-uadltg-itrwcdadvn-401[mzukc]\n\
    tinnm-rms-kcfygvcd-688[cmndf]\n\
    crwwv-mixpqfz-doxpp-xkxivpfp-107[tpawu]\n\
    qxdwpopgsdjh-qphzti-itrwcdadvn-999[lenub]\n\
    jqwpihizlwca-lgm-abwziom-538[iwalm]\n\
    votubcmf-dboez-dpbujoh-gjobodjoh-909[szlxy]\n\
    nwzekwypera-oywrajcan-dqjp-iwjwcaiajp-446[awjpc]\n\
    lxuxaodu-vrurcjah-pajmn-ljwmh-cnlqwxuxph-329[uxahj]\n\
    gvaaz-ezf-efwfmpqnfou-779[scdpt]\n\
    jsvagsulanw-hdsklau-yjskk-vwhdgqewfl-190[saklw]\n\
    yrwxefpi-fewoix-irkmriivmrk-828[irefk]\n\
    jrncbavmrq-rtt-fgbentr-819[rtbna]\n\
    tpspahyf-nyhkl-msvdly-klwsvftlua-409[lsyaf]\n\
    veqtekmrk-tpewxmg-kveww-qerekiqirx-100[szdiy]\n\
    ykhknbqh-ydkykhwpa-hkceopeyo-108[khyeo]\n\
    gifavtkzcv-treup-tfrkzex-dribvkzex-503[ekrtv]\n\
    hafgnoyr-pubpbyngr-bcrengvbaf-351[bgnra]\n\
    ide-htrgti-gpqqxi-gtprfjxhxixdc-999[ixgtd]\n\
    yhtwhnpun-ipvohghykvbz-wshzapj-nyhzz-zavyhnl-617[hznyp]\n\
    enqvbnpgvir-pynffvsvrq-rtt-erprvivat-559[vrnpt]\n\
    jxdkbqfz-yrkkv-pxibp-159[kbpxd]\n\
    etyyx-rbzudmfdq-gtms-rdquhbdr-833[drbmq]\n\
    owshgfarwv-udskkaxawv-hdsklau-yjskk-ogjckzgh-398[kasgh]\n\
    xst-wigvix-gerhc-irkmriivmrk-828[ilntc]\n\
    ugfkmewj-yjsvw-wyy-klgjsyw-684[wyjgk]\n\
    zloolpfsb-mixpqfz-doxpp-pbosfzbp-211[topig]\n\
    fruurvlyh-vfdyhqjhu-kxqw-orjlvwlfv-569[vfhlr]\n\
    xst-wigvix-fyrrc-vigimzmrk-516[irgmv]\n\
    rnqnyfwd-lwfij-wfggny-wjxjfwhm-281[wfjng]\n\
    rdchjbtg-vgpst-ytaanqtpc-sthxvc-557[tcagh]\n\
    fubrjhqlf-fdqgb-frdwlqj-dftxlvlwlrq-465[flqdr]\n\
    qlm-pbzobq-pzxsbkdbo-erkq-xznrfpfqflk-679[bqfkp]\n\
    ltpedcxots-rpcsn-bpcpvtbtci-921[dtejs]\n\
    froruixo-edvnhw-ghsorbphqw-231[horwb]\n\
    bjfutsneji-hmthtqfyj-fsfqdxnx-333[fjthn]\n\
    yhtwhnpun-lnn-zavyhnl-669[wpsgy]\n\
    dmpuamofuhq-ngzzk-xmnadmfadk-742[madfk]\n\
    ejpanjwpekjwh-ywjzu-oanreyao-498[yzjwm]\n\
    eza-dpncpe-qwzhpc-afcnsldtyr-353[ivxnu]\n\
    qekrixmg-nippcfier-gywxsqiv-wivzmgi-464[yxkwm]\n\
    avw-zljyla-ibuuf-ylzlhyjo-383[lyaju]\n\
    lqwhuqdwlrqdo-mhoobehdq-rshudwlrqv-621[qdhlo]\n\
    qvbmzvibqwvit-jcvvg-apqxxqvo-200[vqbix]\n\
    ugjjgkanw-esyfwlau-jsttal-ugflsafewfl-164[fgcep]\n\
    shoewudys-isqludwuh-xkdj-ijehqwu-504[stjyd]\n\
    luxciuwncpy-vohhs-yhachyylcha-214[hcyal]\n\
    gifavtkzcv-sleep-ivjvrity-685[vieta]\n\
    rzvkjiduzy-xviyt-yzqzgjkhzio-161[ziyjk]\n\
    iehepwnu-cnwza-ykjoqian-cnwza-ywjzu-ykwpejc-iwjwcaiajp-316[wajci]\n\
    sorozgxe-mxgjk-jek-vaxingyotm-956[goxej]\n\
    dmpuamofuhq-dmnnuf-dqmocgueufuaz-560[umdfa]\n\
    hjgbwuladw-kusnwfywj-zmfl-ugflsafewfl-450[aezbn]\n\
    esyfwlau-usfvq-ugslafy-ghwjslagfk-294[fsagl]\n\
    shmml-sybjre-erfrnepu-195[ngkjp]\n\
    jlidywncfy-ohmnuvfy-wuhxs-wiuncha-ijyluncihm-240[xtjsm]\n\
    ixeumktoi-lruckx-aykx-zkyzotm-436[kximo]\n\
    nzydfxpc-rclop-upwwjmply-xlylrpxpye-535[plyxc]\n\
    fodvvlilhg-sodvwlf-judvv-pdunhwlqj-725[krngz]\n\
    xjmmjndqz-ezggtwzvi-adivixdib-733[idzgj]\n\
    pbybeshy-pnaql-pbngvat-znantrzrag-533[anbpg]\n\
    fnjyxwrinm-ljwmh-lxjcrwp-bqryyrwp-329[rwjyl]\n\
    lhkhszqx-fqzcd-okzrshb-fqzrr-cdoknxldms-391[zdhkq]\n\
    pynffvsvrq-ohaal-znantrzrag-637[anrfv]\n\
    hafgnoyr-sybjre-genvavat-767[ngacu]\n\
    lhkhszqx-fqzcd-bgnbnkzsd-lzmzfdldms-443[ynael]\n\
    lugjuacha-wbiwifuny-mufym-786[uafim]\n\
    vkrhzxgbv-xzz-ftgtzxfxgm-995[xzgft]\n\
    uzfqdzmfuazmx-rxaiqd-emxqe-170[mqxza]\n\
    ajvyjprwp-snuuhknjw-anlnrerwp-771[njprw]\n\
    zuv-ykixkz-igtje-iugzotm-zxgototm-930[lifhb]\n\
    mfklstdw-wyy-ksdwk-294[kwdsy]\n\
    kyelcrga-slqryzjc-hcjjwzcyl-qrmpyec-990[kypqm]\n\
    vkppo-sxesebqju-tulubefcudj-400[uebjp]\n\
    ynukcajey-xqjju-lqnydwoejc-394[jycen]\n\
    qzlozfhmf-qzchnzbshud-qzaahs-qdzbpthrhshnm-287[hzqsa]\n\
    kmjezxodgz-mvwwdo-ncdkkdib-109[dkmow]\n\
    oazegyqd-sdmpq-ngzzk-bgdotmeuzs-482[zdgem]\n\
    qfkkj-mldvpe-cpdplcns-561[pcdkl]\n\
    hvbizodx-ezggtwzvi-xjiovdihzio-577[voqzy]\n\
    iuxxuyobk-inuiurgzk-xkikobotm-722[ikuox]\n\
    jyddc-gsvvswmzi-jpsaiv-jmrergmrk-958[cpedy]\n\
    vhkkhlbox-unggr-hixktmbhgl-449[hgkbl]\n\
    clotzlnetgp-nsznzwlep-dpcgtnpd-145[tehzy]\n\
    plolwdub-judgh-hjj-dqdobvlv-543[zkryh]\n\
    ajmrxjlcren-ouxfna-ydalqjbrwp-355[ajrln]\n\
    uqtqbizg-ozilm-lgm-amzdqkma-304[mqzag]\n\
    lnkfaypeha-zua-lqnydwoejc-914[aelny]\n\
    ibghopzs-pwcvonofrcig-qobrm-aobousasbh-844[obsac]\n\
    ocipgvke-eqpuwogt-itcfg-tcddkv-gpikpggtkpi-804[salbg]\n\
    ajmrxjlcren-lqxlxujcn-uxprbcrlb-823[lrxcj]\n\
    rgndvtcxr-ytaanqtpc-sthxvc-843[tcanr]\n\
    hqcfqwydw-sxesebqju-qdqboiyi-894[qbdei]\n\
    tbxmlkfwba-gbiivybxk-qbzeklildv-757[biklv]\n\
    vetllbybxw-utldxm-vhgmtbgfxgm-735[bglmt]\n\
    mfklstdw-hdsklau-yjskk-vwhsjlewfl-528[xyuts]\n\
    pxtihgbsxw-vtgwr-nlxk-mxlmbgz-657[xgblm]\n\
    bnmrtldq-fqzcd-dff-lzqjdshmf-677[szdpt]\n\
    xekdwvwnzkqo-sawlkjevaz-fahhuxawj-nayaerejc-654[zdeyh]\n\
    gzefmnxq-bxmefuo-sdmee-xmnadmfadk-170[medfx]\n\
    gpsxdprixkt-eaphixr-vgphh-rjhidbtg-htgkxrt-115[hgprt]\n\
    eza-dpncpe-hplazytkpo-awldetn-rcldd-xlcvpetyr-535[tnpmg]\n\
    bnmrtldq-fqzcd-eknvdq-sdbgmnknfx-781[dnqbf]\n\
    nzcczdtgp-dnlgpyrpc-sfye-opgpwzaxpye-899[pcgyz]\n\
    nwzekwypera-ydkykhwpa-odellejc-992[pwqrh]\n\
    oknkvcta-itcfg-gii-wugt-vguvkpi-154[giktv]\n\
    tcrjjzwzvu-upv-kvtyefcfxp-373[vcfjp]\n\
    xst-wigvix-ikk-stivexmsrw-230[isxkt]\n\
    fkqbokxqflkxi-zelzlixqb-qoxfkfkd-705[mntlq]\n\
    qlm-pbzobq-yxphbq-zrpqljbo-pbosfzb-237[bpqoz]\n\
    drxevkzt-avccpsvre-wzeretzex-269[ervzc]\n\
    ksodcbwnsr-pogysh-oqeiwgwhwcb-480[mxdsl]\n\
    tyepcyletzylw-qwzhpc-xlylrpxpye-613[tvcgy]\n\
    rnqnyfwd-lwfij-hmthtqfyj-qtlnxynhx-437[ukdrt]\n\
    oxjmxdfkd-jfifqxov-doxab-yxphbq-ixyloxqlov-393[xodfq]\n\
    tcorcikpi-ejqeqncvg-fgrnqaogpv-804[cgqei]\n\
    hqtyeqsjylu-zubboruqd-qsgkyiyjyed-712[yqubd]\n\
    hvbizodx-ezggtwzvi-gjbdnodxn-967[dgzbi]\n\
    zntargvp-enoovg-fgbentr-923[gneor]\n\
    mvydjvxodqz-wvnfzo-hvmfzodib-447[zfpes]\n\
    emixwvqhml-zijjqb-lmdmtwxumvb-148[mbijl]\n\
    frqvxphu-judgh-gbh-whfkqrorjb-179[jwmgn]\n\
    kyelcrga-njyqrga-epyqq-dglylagle-782[glyae]\n\
    npmhcargjc-pyzzgr-bctcjmnkclr-522[crgjm]\n\
    pxtihgbsxw-xzz-tvjnblbmbhg-943[bxght]\n\
    oknkvcta-itcfg-hnqygt-qrgtcvkqpu-206[tcgkq]\n\
    amlqskcp-epybc-zyqicr-sqcp-rcqrgle-522[bnemi]\n\
    enqvbnpgvir-pnaql-fuvccvat-299[vnacp]\n\
    xlrypetn-mldvpe-cpnptgtyr-509[nugrq]\n\
    mbggf-qlssfilhu-klclsvwtlua-383[lsfgu]\n\
    wlsiayhcw-wuhxs-wiuncha-mniluay-656[wahiu]\n\
    gvaaz-cbtlfu-efqmpznfou-415[byzhx]\n\
    rzvkjiduzy-nxvqzibzm-cpio-hvivbzhzio-343[zivbh]\n\
    bqxnfdmhb-dff-otqbgzrhmf-781[fbdhm]\n\
    wihmogyl-aluxy-wuhxs-wiuncha-lyuwkocmcncih-838[chuwi]\n\
    enzcntvat-onfxrg-hfre-grfgvat-689[nkvyi]\n\
    xgjougizobk-igtje-iugzotm-ygrky-540[giojk]\n\
    mbggf-kfl-aljouvsvnf-773[fglva]\n\
    qzoggwtwsr-xszzmpsob-sbuwbssfwbu-662[sbwzg]\n\
    wsvsdkbi-qbkno-mkxni-mykdsxq-nofovyzwoxd-744[kodns]\n\
    tpspahyf-nyhkl-lnn-klzpnu-721[nlphk]\n\
    pejji-nio-bomosfsxq-380[oijsb]\n\
    amlqskcp-epybc-cee-nspafyqgle-132[dsayt]\n\
    luxciuwncpy-xsy-lyuwkocmcncih-240[cuyil]\n\
    irdgrxzex-gcrjkzt-xirjj-vexzevvizex-165[xerzi]\n\
    lxaaxbren-snuuhknjw-ujkxajcxah-381[axjnu]\n\
    ktfitzbgz-ynssr-xzz-wxiehrfxgm-839[sjagq]\n\
    hafgnoyr-enzcntvat-pnaql-pbngvat-genvavat-975[antvg]\n\
    dfcxsqhwzs-qobrm-fsqswjwbu-896[sqwbf]\n\
    zsxyfgqj-xhfajsljw-mzsy-xytwflj-619[jfsxy]\n\
    yhwooebeaz-ywjzu-ykwpejc-hkceopeyo-706[eoywc]\n\
    mvydjvxodqz-wpiit-kpmxcvndib-863[yrjnz]\n\
    otzkxtgzoutgr-igtje-iugzotm-iutzgotsktz-332[tgzoi]\n\
    pdjqhwlf-sodvwlf-judvv-vdohv-855[vdfhj]\n\
    gpewwmjmih-fyrrc-gywxsqiv-wivzmgi-724[iwgmr]\n\
    oqnidbshkd-idkkxadzm-rsnqzfd-391[lpscd]\n\
    rgndvtcxr-hrpktcvtg-wjci-uxcpcrxcv-765[crtvx]\n\
    esyfwlau-tmffq-xafsfuafy-970[fasuy]\n\
    gvaaz-dipdpmbuf-efqbsunfou-545[zmynh]\n\
    zsxyfgqj-wfggny-uzwhmfxnsl-463[cvqjn]\n\
    oazegyqd-sdmpq-ngzzk-emxqe-430[flhis]\n\
    jvuzbtly-nyhkl-zjhclunly-obua-zlycpjlz-643[ueimk]\n\
    surmhfwloh-vfdyhqjhu-kxqw-ghsorbphqw-205[hqwfo]\n\
    pualyuhapvuhs-zjhclunly-obua-thuhnltlua-825[ficqs]\n\
    wbhsfbohwcboz-rms-oqeiwgwhwcb-194[wbhoc]\n\
    gpsxdprixkt-ytaanqtpc-bpcpvtbtci-635[qkzhc]\n\
    rnqnyfwd-lwfij-xhfajsljw-mzsy-qfgtwfytwd-931[fwjyd]\n\
    ubhatstkwhnl-ktuubm-vnlmhfxk-lxkobvx-787[gtsqv]\n\
    lqwhuqdwlrqdo-udeelw-dftxlvlwlrq-413[ldqwe]\n\
    kloqemlib-lygbzq-pqloxdb-991[lbqod]\n\
    veqtekmrk-tpewxmg-kveww-hiwmkr-282[ekwmr]\n\
    rflsjynh-ytu-xjhwjy-jll-knsfshnsl-333[jlshn]\n\
    bknsykmdsfo-pvygob-domrxyvyqi-432[yobdk]\n\
    mybbycsfo-bkllsd-kxkvicsc-822[yzxcq]\n\
    zixppfcfba-yxphbq-absbilmjbkq-991[gbhts]\n\
    udskkaxawv-uzgugdslw-klgjsyw-684[gksuw]\n\
    clxalrtyr-mfyyj-qtylyntyr-665[ylrta]\n\
    uiovmbqk-jiasmb-lmaqov-694[mabio]\n\
    xmtjbzidx-mvwwdo-zibdizzmdib-161[dizbm]\n\
    wyvqljapsl-jovjvshal-bzly-alzapun-643[lajvp]\n\
    zlilocri-zxkav-mrozexpfkd-445[rifng]\n\
    pinovwgz-zbb-hvmfzodib-811[bziov]\n\
    rtqlgevkng-dwppa-vgejpqnqia-284[gpqae]\n\
    vrurcjah-pajmn-ljwmh-jlzdrbrcrxw-667[lmdrk]\n\
    jlidywncfy-dyffsvyuh-ijyluncihm-838[yficd]\n\
    cebwrpgvyr-sybjre-qrirybczrag-741[tsrqd]\n\
    pbafhzre-tenqr-onfxrg-grpuabybtl-949[rbaef]\n\
    lahxpnwrl-ljwmh-lxjcrwp-nwprwnnarwp-433[seonp]\n\
    iuxxuyobk-igtje-iugzotm-iutzgotsktz-644[tiugo]\n\
    qfkkj-prr-fdpc-epdetyr-951[prdef]\n\
    nchhg-akidmvomz-pcvb-nqvivkqvo-954[osgtz]\n\
    htwwtxnaj-gzssd-ijufwyrjsy-385[jswty]\n\
    myxcewob-qbkno-mrymyvkdo-bokmaescsdsyx-328[sezot]\n\
    rzvkjiduzy-mvwwdo-ncdkkdib-499[mfyze]\n\
    emixwvqhml-lgm-uizsmbqvo-798[milqv]\n\
    xmtjbzidx-wvnfzo-vivgtndn-941[nvdit]\n\
    bknsykmdsfo-zvkcdsm-qbkcc-cobfsmoc-198[cksbm]\n\
    gsrwyqiv-kvehi-fyrrc-jmrergmrk-906[regik]\n\
    bqvvu-oywrajcan-dqjp-nawymqeoepekj-524[aejqn]\n\
    upq-tfdsfu-sbccju-bobmztjt-883[btucf]\n\
    surmhfwloh-fruurvlyh-mhoobehdq-orjlvwlfv-933[zymnj]\n\
    wkqxodsm-tovvilokx-gybucryz-588[okvxy]\n\
    nchhg-ntwemz-twoqabqka-902[ahnqt]\n\
    iqmbazulqp-rgllk-rxaiqd-fqotzaxask-950[dtanc]\n\
    ejpanjwpekjwh-nwxxep-bejwjyejc-732[jewpn]\n\
    ajmrxjlcren-fnjyxwrinm-lqxlxujcn-vjwjpnvnwc-329[yhgwz]\n\
    qcbgiasf-ufors-qvcqczohs-aofyshwbu-532[scfoq]\n\
    jsehsyafy-vqw-ugflsafewfl-970[mfzcn]\n\
    fab-eqodqf-qss-geqd-fqefuzs-560[qfesd]\n\
    jef-iushuj-uww-sedjqydcudj-322[qyadz]\n\
    kfg-jvtivk-jtrmvexvi-ylek-ivtvzmzex-347[wmlfu]\n\
    pxtihgbsxw-ietlmbv-zktll-wxiehrfxgm-371[xiltb]\n\
    wfintfhynaj-hmthtqfyj-jslnsjjwnsl-463[jnfhs]\n\
    forwcoqhwjs-foppwh-kcfygvcd-480[cfowh]\n\
    kzgwomvqk-rmttgjmiv-camz-bmabqvo-616[mvabg]\n\
    pybgmyargtc-aylbw-amyrgle-nspafyqgle-392[yaglb]\n\
    jyfvnlupj-ihzrla-lunpullypun-149[lytps]\n\
    dpmpsgvm-cbtlfu-xpsltipq-467[plmst]\n\
    oxaflxzqfsb-avb-zrpqljbo-pbosfzb-965[bfoza]\n\
    amlqskcp-epybc-afmamjyrc-bctcjmnkclr-392[cmabj]\n\
    encuukhkgf-tcddkv-yqtmujqr-362[kucdq]\n\
    lqwhuqdwlrqdo-edvnhw-hqjlqhhulqj-595[sywmh]\n\
    njmjubsz-hsbef-qspkfdujmf-cbtlfu-vtfs-uftujoh-857[fujsb]\n\
    dsxxw-djmucp-umpiqfmn-340[mdpux]\n\
    aflwjfslagfsd-hdsklau-yjskk-dstgjslgjq-736[sjlad]\n\
    ynukcajey-ywjzu-iwjwcaiajp-758[sthmn]\n\
    froruixo-lqwhuqdwlrqdo-fdqgb-uhdftxlvlwlrq-621[gtcry]\n\
    chnylhuncihuf-jfumncw-alumm-mylpcwym-526[tyodr]\n\
    ujoon-qphzti-uxcpcrxcv-817[copux]\n\
    bkwzkqsxq-cmkfoxqob-rexd-ckvoc-666[kcoqx]\n\
    pelbtravp-enqvbnpgvir-qlr-qrfvta-403[wqynx]\n\
    yhtwhnpun-ibuuf-thuhnltlua-643[trfed]\n\
    willimcpy-yaa-wihnuchgyhn-344[hiyac]\n\
    thnulapj-jovjvshal-mpuhujpun-799[juhpa]\n\
    nzcczdtgp-mldvpe-lnbftdtetzy-821[qvnmi]\n\
    qzlozfhmf-idkkxadzm-btrsnldq-rdquhbd-209[dqzbf]\n\
    ajvyjprwp-ouxfna-vjwjpnvnwc-407[jnpvw]\n\
    hcd-gsqfsh-dzoghwq-ufogg-zcuwghwqg-688[nwgox]\n\
    jrncbavmrq-qlr-genvavat-169[arvnq]\n\
    crwwv-yrkkv-jxkxdbjbkq-653[ylpzs]\n\
    pejji-tovvilokx-vyqscdsmc-146[vcijo]\n\
    ikhcxvmbex-lvtoxgzxk-angm-inkvatlbgz-189[xgkva]\n\
    jyddc-wgezirkiv-lyrx-xiglrspskc-620[ircdg]\n\
    ajyqqgdgcb-aylbw-amyrgle-rpyglgle-210[glyab]\n\
    mhi-lxvkxm-yehpxk-kxvxbobgz-319[lcest]\n\
    rkpqxyib-gbiivybxk-lmboxqflkp-211[vustn]\n\
    jchipqat-rdadguja-hrpktcvtg-wjci-tcvxcttgxcv-999[ctagj]\n\
    ovbunmneqbhf-fpniratre-uhag-grpuabybtl-949[banru]\n\
    nchhg-xtiabqk-oziaa-abwziom-174[aibho]\n\
    dwbcjkun-ljwmh-ydalqjbrwp-303[jwbdl]\n\
    lxuxaodu-krxqjijamxdb-bljenwpna-qdwc-bnaerlnb-563[ycjlt]\n\
    yhkpvhjapcl-wshzapj-nyhzz-shivyhavyf-461[hyapv]\n\
    udglrdfwlyh-mhoobehdq-wudlqlqj-959[dlhqo]\n\
    myxcewob-qbkno-lsyrkjkbnyec-lexxi-cdybkqo-588[nfdem]\n\
    fnjyxwrinm-ljwmh-lxwcjrwvnwc-459[wjncl]\n\
    udpsdjlqj-udeelw-ghvljq-491[djleq]\n\
    zloolpfsb-yrkkv-mrozexpfkd-783[koflp]\n\
    drxevkzt-jtrmvexvi-ylek-ivrthlzjzkzfe-997[evzkr]\n\
    ykjoqian-cnwza-ywjzu-ykwpejc-opknwca-264[wacjk]\n\
    xekdwvwnzkqo-zua-zalwnpiajp-992[awzkn]\n\
    fydelmwp-aczupnetwp-prr-cplnbftdtetzy-847[ptecd]\n\
    lxwbdvna-pajmn-ouxfna-anjlzdrbrcrxw-563[anrxb]\n\
    xfbqpojafe-gmpxfs-tbmft-545[rgdzm]\n\
    kzeed-ojqqdgjfs-ijuqtdrjsy-411[jdqes]\n\
    ktiaaqnqml-ntwemz-uizsmbqvo-642[azvew]\n\
    udpsdjlqj-gbh-whfkqrorjb-725[rnqmt]\n\
    lahxpnwrl-ouxfna-ydalqjbrwp-745[alnpr]\n\
    dsxxw-bwc-mncpyrgmlq-548[cmwxb]\n\
    joufsobujpobm-gvaaz-qmbtujd-hsbtt-fohjoffsjoh-727[ojbfh]\n\
    laffe-jek-ynovvotm-670[efova]\n\
    nzcczdtgp-ojp-xlcvpetyr-353[nvmak]\n\
    kgjgrypw-epybc-djmucp-qyjcq-496[sqhmg]\n\
    ykjoqian-cnwza-ywjzu-klanwpekjo-680[ajknw]\n\
    nbhofujd-dipdpmbuf-efqbsunfou-415[fubdn]\n\
    oaddaeuhq-ngzzk-fqotzaxask-144[hbxcm]\n\
    lujbbrornm-vjpwncrl-snuuhknjw-ldbcxvna-bnaerln-459[nblrj]\n\
    xgvnndadzy-wvnfzo-yzkvmohzio-135[znovd]\n\
    jchipqat-tvv-jhtg-ithixcv-271[ymstr]\n\
    xtwtelcj-rclop-nlyoj-dstaatyr-431[ntags]\n\
    iutyaskx-mxgjk-jek-xkykgxin-618[kxgij]\n\
    pxtihgbsxw-ktwbhtvmbox-cxeeruxtg-tgterlbl-943[txbeg]\n\
    xfbqpojafe-dboez-dpbujoh-mbcpsbupsz-441[bpode]\n\
    qfkkj-mfyyj-opalcexpye-613[yefjk]\n\
    ejpanjwpekjwh-lhwopey-cnwoo-nawymqeoepekj-836[ewjop]\n\
    qjopwxha-xqjju-odellejc-732[jeloq]\n\
    bnknqetk-atmmx-lzmzfdldms-261[mdkln]\n\
    xgsvgmotm-pkrrehkgt-sgtgmksktz-332[gktms]\n\
    ryexqpqhteki-vbemuh-huqsgkyiyjyed-244[tjsqx]\n\
    xjinphzm-bmvyz-ytz-yzkvmohzio-239[oznyv]\n\
    eqttqukxg-rncuvke-itcuu-cpcnauku-180[jztvf]\n\
    xgjougizobk-xghhoz-ygrky-696[gohkx]\n\
    mtzslklcozfd-nsznzwlep-opgpwzaxpye-769[zplen]\n\
    kgjgrypw-epybc-pyzzgr-jmegqrgaq-626[gprye]\n\
    jlidywncfy-wbiwifuny-mniluay-396[iynwf]\n\
    myvybpev-mkxni-mykdsxq-psxkxmsxq-536[rxnml]\n\
    ibghopzs-qobrm-rsdzcmasbh-246[bshmo]\n\
    jyfvnlupj-wshzapj-nyhzz-klwsvftlua-201[jlzaf]\n\
    dzczkrip-xiruv-avccpsvre-cfxzjkztj-113[czrvi]\n\
    dmbttjgjfe-dboez-dpbujoh-nbslfujoh-493[bjdoe]\n\
    kfg-jvtivk-gifavtkzcv-srjbvk-jvimztvj-347[aymns]\n\
    ktwbhtvmbox-vetllbybxw-ietlmbv-zktll-wxlbzg-241[bltvw]\n\
    tmrszakd-cxd-qdrdzqbg-417[dqrza]\n\
    nzydfxpc-rclop-nsznzwlep-nfdezxpc-dpcgtnp-899[ynbfk]\n\
    fruurvlyh-mhoobehdq-rshudwlrqv-491[hrudl]\n\
    odkasqzuo-pkq-emxqe-144[qekoa]\n\
    hwbba-dcumgv-vtckpkpi-180[bckpv]\n\
    lsyrkjkbnyec-cmkfoxqob-rexd-wkbuodsxq-718[yfzcq]\n\
    xgvnndadzy-kgvnodx-bmvnn-xjiovdihzio-395[ndvio]\n\
    willimcpy-vumeyn-omyl-nymncha-890[nmyux]\n\
    mbggf-zjhclunly-obua-yljlpcpun-487[lubcg]\n\
    ryexqpqhteki-sqdto-tuiywd-608[xhjzp]\n\
    egdytrixat-gpqqxi-prfjxhxixdc-193[xidgp]\n\
    nbhofujd-dszphfojd-cbtlfu-sftfbsdi-909[fdbsh]\n\
    rflsjynh-gfxpjy-htsyfnsrjsy-489[istpm]\n\
    vkrhzxgbv-vetllbybxw-ietlmbv-zktll-phkdlahi-189[lbvhk]\n\
    lujbbrornm-ouxfna-uxprbcrlb-459[ozvca]\n\
    pbafhzre-tenqr-fpniratre-uhag-erprvivat-117[raept]\n\
    xgvnndadzy-xjmmjndqz-ytz-nvgzn-577[nzdgj]\n\
    houngfgxjuay-pkrrehkgt-jkyomt-618[dltyf]\n\
    bjfutsneji-hmthtqfyj-jslnsjjwnsl-411[jsntf]\n\
    sgmtkzoi-lruckx-rghuxgzuxe-774[guxkr]\n\
    nuatmlmdpage-nmewqf-abqdmfuaze-326[zbewa]\n\
    dsxxw-zsllw-qyjcq-912[lqswx]\n\
    cvabijtm-kivlg-kwibqvo-aitma-226[mvuhw]\n\
    yuxufmdk-sdmpq-qss-bgdotmeuzs-768[sdmuq]\n\
    qspkfdujmf-fhh-usbjojoh-597[elvgu]\n\
    htqtwkzq-gfxpjy-hzxytrjw-xjwanhj-359[jpytc]\n\
    gbc-frperg-pnaql-pbngvat-grpuabybtl-169[bgpar]\n\
    yhkpvhjapcl-qlssfilhu-klclsvwtlua-123[lhsac]\n\
    nglmtuex-vtgwr-vhtmbgz-ftkdxmbgz-813[emnca]\n\
    atyzghrk-lruckx-ygrky-592[kryga]\n\
    dkqjcbctfqwu-tcddkv-tgegkxkpi-362[kcdtg]\n\
    sawlkjevaz-bhksan-lqnydwoejc-914[aejkl]\n\
    kdijqrbu-uww-ijehqwu-712[uwijq]\n\
    rwcnawjcrxwju-snuuhknjw-mnbrpw-121[wnjru]\n\
    vrurcjah-pajmn-mhn-bcxajpn-225[ajnch]\n\
    clotzlnetgp-mldvpe-cpdplcns-353[lpcde]\n\
    wihmogyl-aluxy-dyffsvyuh-womnigyl-mylpcwy-682[khzto]\n\
    qcbgiasf-ufors-dzoghwq-ufogg-kcfygvcd-428[mselb]\n\
    kwtwznct-ktiaaqnqml-akidmvomz-pcvb-bmkpvwtwog-824[kmtwa]\n\
    crwwv-oxyyfq-pqloxdb-289[oqwxy]\n\
    iutyaskx-mxgjk-ckgvutofkj-lruckx-jkvgxzsktz-852[kxgjt]\n\
    irgyyolokj-yigbktmkx-natz-giwaoyozout-930[oygik]\n\
    yhtwhnpun-buzahisl-jhukf-jvhapun-mpuhujpun-565[uhnpj]\n\
    fbebmtkr-zktwx-ktuubm-ybgtgvbgz-761[ifsyt]\n\
    lejkrscv-avccpsvre-dribvkzex-165[vcerk]\n\
    avw-zljyla-yhiipa-jvuahputlua-617[aluhi]\n\
    xjmmjndqz-wpiit-xjiovdihzio-889[ijdmo]\n\
    pbybeshy-fpniratre-uhag-jbexfubc-819[snuje]\n\
    ktwbhtvmbox-vtgwr-phkdlahi-683[htbkv]\n\
    avw-zljyla-wshzapj-nyhzz-ylzlhyjo-409[zlyah]\n\
    tcrjjzwzvu-treup-tfrkzex-drerxvdvek-633[retvz]\n\
    zlkprjbo-doxab-ciltbo-qoxfkfkd-627[nmwxj]\n\
    vkrhzxgbv-lvtoxgzxk-angm-labiibgz-995[gbvxz]\n\
    gpbepvxcv-hrpktcvtg-wjci-sthxvc-609[cvptg]\n\
    hwbba-gii-fgrctvogpv-804[gbiva]\n\
    nwlddtqtpo-nsznzwlep-dstaatyr-275[tdnal]\n\
    ovbunmneqbhf-fpniratre-uhag-hfre-grfgvat-637[fraeg]\n\
    pkl-oaynap-fahhuxawj-nayaerejc-888[mnchz]\n\
    etaqigpke-hnqygt-qrgtcvkqpu-752[antmz]\n\
    lugjuacha-wuhxs-ijyluncihm-214[hfsun]\n\
    zloolpfsb-zxkav-abpfdk-341[abfkl]\n\
    bknsykmdsfo-mkxni-mykdsxq-bokmaescsdsyx-822[skmdx]\n\
    xjgjmapg-zbb-hvivbzhzio-499[bzghi]\n\
    amjmpdsj-ajyqqgdgcb-qaytclecp-fslr-qrmpyec-652[smgnt]\n\
    qczcftiz-dzoghwq-ufogg-rsgwub-714[gzcfo]\n\
    gokzyxsjon-wsvsdkbi-qbkno-nio-myxdksxwoxd-900[oksxd]\n\
    ktfitzbgz-vtgwr-vhtmbgz-lxkobvxl-969[tbgvz]\n\
    hqtyeqsjylu-uww-efuhqjyedi-270[dytgj]\n\
    ovbunmneqbhf-pnaql-pbngvat-freivprf-845[nbfpv]\n\
    kzgwomvqk-akidmvomz-pcvb-aitma-902[makvi]\n\
    sedikcuh-whqtu-zubboruqd-bqrehqjeho-790[hqube]\n\
    kwvacumz-ozilm-jcvvg-lmaqov-382[vmacl]\n\
    rgndvtcxr-qjccn-prfjxhxixdc-661[cxrdj]\n\
    tbxmlkfwba-oxyyfq-tlohpelm-523[uhvaf]\n\
    iuxxuyobk-inuiurgzk-vaxingyotm-514[iuxgk]\n\
    odkasqzuo-omzpk-oamfuzs-ymzmsqyqzf-170[zmoqs]\n\
    ktwbhtvmbox-unggr-lxkobvxl-527[bxgkl]\n\
    ynssr-vtgwr-mxvaghehzr-423[rghsv]\n\
    sgmtkzoi-kmm-lotgtiotm-670[mtogi]\n\
    ugdgjxmd-vqw-kwjnauwk-944[wdgjk]\n\
    rdggdhxkt-jchipqat-snt-gthtpgrw-713[tghdp]\n\
    zvyvgnel-tenqr-wryylorna-npdhvfvgvba-663[vnrya]\n\
    jsvagsulanw-usfvq-ugslafy-esjcwlafy-762[fygle]\n\
    zsxyfgqj-wfggny-hzxytrjw-xjwanhj-931[jgwxy]\n\
    wkqxodsm-tovvilokx-dbksxsxq-822[cpsgv]\n\
    mbiyqoxsm-tovvilokx-yzobkdsyxc-458[oxybi]\n\
    lzfmdshb-bzmcx-bnzshmf-vnqjrgno-261[ujfyc]\n\
    oknkvcta-itcfg-dcumgv-hkpcpekpi-258[ckpgi]\n\
    iuxxuyobk-inuiurgzk-sgtgmksktz-202[ztjvk]\n\
    hjgbwuladw-hdsklau-yjskk-ljsafafy-372[mvkts]\n\
    rdadguja-eaphixr-vgphh-tcvxcttgxcv-739[acght]\n\
    nsyjwsfyntsfq-idj-xmnuunsl-983[nsfju]\n\
    gifavtkzcv-wcfnvi-rthlzjzkzfe-971[zfvci]\n\
    qvbmzvibqwvit-jiasmb-zmamizkp-278[imbvz]\n\
    houngfgxjuay-ckgvutofkj-yigbktmkx-natz-ykxboiky-930[kgyot]\n\
    guahyncw-wuhxs-lyuwkocmcncih-500[chuwn]\n\
    gpbepvxcv-ytaanqtpc-bpgztixcv-479[pctva]\n\
    ksodcbwnsr-pogysh-gsfjwqsg-584[sgowb]\n\
    dmybmsuzs-pkq-mzmxkeue-612[meksu]\n\
    xmtjbzidx-xviyt-vxlpdndodji-577[dxijt]\n\
    dpmpsgvm-kfmmzcfbo-bobmztjt-701[mbfop]\n\
    oxjmxdfkd-pzxsbkdbo-erkq-xznrfpfqflk-627[jhlvw]\n\
    npmhcargjc-aylbw-amyrgle-pcacgtgle-652[acgle]\n\
    votubcmf-fhh-tfswjdft-701[fthbc]\n\
    gzefmnxq-dmnnuf-geqd-fqefuzs-482[fenqd]\n\
    dmpuamofuhq-oazegyqd-sdmpq-eomhqzsqd-tgzf-emxqe-378[mctsn]\n\
    gzefmnxq-dmnnuf-etubbuzs-508[nubef]\n\
    bwx-amkzmb-ntwemz-ivitgaqa-902[noeig]\n\
    rdchjbtg-vgpst-qphzti-pcpanhxh-635[hptcg]\n\
    qzchnzbshud-atmmx-sqzhmhmf-131[hmzqs]\n\
    tipfxvezt-treup-tfrkzex-cfxzjkztj-633[tzefx]\n\
    mvhkvbdib-wpiit-vivgtndn-603[ivbdn]\n\
    hqfxxnknji-ojqqdgjfs-htsyfnsrjsy-957[bdtai]\n\
    clxalrtyr-clmmte-dezclrp-275[lcrem]\n\
    fubrjhqlf-udpsdjlqj-udeelw-ghyhorsphqw-387[hdjlq]\n\
    qzoggwtwsr-dzoghwq-ufogg-hsqvbczcum-792[goqwz]\n\
    nvrgfezqvu-sleep-ivtvzmzex-113[evzfg]\n\
    fruurvlyh-iorzhu-orjlvwlfv-881[jcdtf]\n\
    myxcewob-qbkno-mkxni-mykdsxq-nozvyiwoxd-900[oxkmn]\n\
    wyvqljapsl-jhukf-jvhapun-aljouvsvnf-955[jvalu]\n\
    tpspahyf-nyhkl-jhukf-zhslz-643[pytob]\n\
    kgjgrypw-epybc-rmn-qcapcr-qaytclecp-fslr-nspafyqgle-938[mxunk]\n\
    yknnkoera-zua-klanwpekjo-446[dphwe]\n\
    kzgwomvqk-xtiabqk-oziaa-bziqvqvo-278[qaiko]\n\
    iruzfrtkzmv-wcfnvi-rercpjzj-555[rzcfi]\n\
    diozmivodjivg-apuut-nxvqzibzm-cpio-yzkvmohzio-447[iozvm]\n\
    fydelmwp-ojp-nzyeltyxpye-405[nyzmg]\n\
    ktiaaqnqml-ziuxioqvo-kivlg-kwibqvo-zmikycqaqbqwv-564[qikva]\n\
    xst-wigvix-ikk-vieguymwmxmsr-724[imxgk]\n\
    jfifqxov-doxab-ciltbo-jxohbqfkd-965[obfxd]\n\
    rdggdhxkt-hrpktcvtg-wjci-rdcipxcbtci-531[ctdgi]\n\
    awzwhofm-ufors-qobrm-qcohwbu-oqeiwgwhwcb-896[mkvzu]\n\
    vagreangvbany-fpniratre-uhag-qrcnegzrag-689[xfuaz]\n\
    vjpwncrl-lqxlxujcn-jlzdrbrcrxw-693[lrcjx]\n\
    ibghopzs-dzoghwq-ufogg-rsdzcmasbh-948[ghosz]\n\
    ynssr-vhglnfxk-zktwx-ietlmbv-zktll-phkdlahi-917[lkhti]\n\
    tfcfiwlc-treup-tfrkzex-dribvkzex-685[efrtc]\n\
    frqvxphu-judgh-fkrfrodwh-hqjlqhhulqj-803[wjnmk]\n\
    vhkkhlbox-xzz-kxtvjnblbmbhg-631[bhkxl]\n\
    ltpedcxots-snt-stepgibtci-297[tscei]\n\
    nzwzcqfw-nlyoj-xlylrpxpye-275[lynpw]\n\
    ejpanjwpekjwh-nwilwcejc-zua-hkceopeyo-498[ypoze]\n\
    sno-rdbqds-azrjds-rghoohmf-859[dorsh]\n\
    qzoggwtwsr-suu-qcbhowbasbh-480[bswgh]\n\
    gvcskirmg-ikk-gywxsqiv-wivzmgi-698[rmvil]\n\
    ktwbhtvmbox-ftzgxmbv-vtgwr-ftkdxmbgz-163[tbgmv]\n\
    oxmeeuruqp-omzpk-oamfuzs-efadmsq-716[meoua]\n\
    xjinphzm-bmvyz-hvbizodx-xviyt-xjvodib-ozxcijgjbt-343[ixbjv]\n\
    jyfvnlupj-ibuuf-svnpzapjz-851[gmsnf]"

main(args: int[][]) {
    input: int[][] = split(INPUT, '\n')
    sum: int = 0

    i: int = 0
    while i < length(input) {
        room: int[] = input[i]

        if realRoom(room) {
            id: int, success: bool = parseInt(slice::<int>(room, length(room) - 10, length(room) - 7))
            assert(success)
            sum = sum + id
        }

        i = i + 1
    }

    println(unparseInt(sum))
}

realRoom(room: int[]): bool {
    counts: VectorMap::<Integer, Integer> = newVectorMap::<Integer, Integer>()

    i: int = 0

    // Count letters in name, ignoring suffix: DDD[AAAAA]
    while i < length(room) - 10 {
        if room[i] != '-' {
            key: Integer = newInteger(room[i])
            value: Integer = counts.get(key)

            if value == null {
                _ = counts.insert(key, newInteger(1))
            } else {
                value.value = value.value + 1
            }
        }

        i = i + 1
    }

    sorted: Vector::<Letter> = newVector::<Letter>()

    // Collect used letters and counts for sorting
    j: int = 0
    while j < counts.size() {
        sorted.push(newLetter(
            counts.keys.get(j).value,
            counts.values.get(j).value
        ))
        j = j + 1
    }

    bubble_sort::<Letter>(sorted)

    k: int = 0
    while k < 5 {
        checksum: int = room[length(room) - 6 + k]

        if sorted.get(k).letter != checksum {
            return false
        }

        k = k + 1
    }

    return true
}

template slice<T>(input: T[], low: int, high: int): T[] {
    output: T[high - low]

    i: int = low
    while i < high {
        output[i - low] = input[i]
        i = i + 1
    }

    return output
}

// Requires:
//
// class T {
//   compare(other: T): int
// }
template bubble_sort<T>(input: Vector::<T>) {
    swapped: bool = true

    while swapped {
        swapped = false

        i: int = 0
        while i + 1 < input.size() {
            if input.get(i).compare(input.get(i + 1)) > 0 {
                input.swap(i, i + 1)
                swapped = true
            }

            i = i + 1
        }
    }
}

class Letter {
    count, letter: int

    compare(other: Letter): int {
        // Sort count in descending order
        order: int = other.count - count

        if order == 0 {
            // And then letter in ascending order
            return letter - other.letter
        } else {
            return order
        }
    }
}

newLetter(letter: int, count: int): Letter {
    letter': Letter = new Letter
    letter'.letter = letter
    letter'.count = count
    return letter'
}

class Integer {
    value: int
    equals(other: Integer): bool {
        return value == other.value
    }
}

newInteger(value: int): Integer {
    integer: Integer = new Integer
    integer.value = value
    return integer
}

split(string: int[], character: int): int[][] {
    count: int = 1

    // First pass: compute number of splits
    i: int = 0
    while i < length(string) {
        if string[i] == character {
            count = count + 1
        }

        i = i + 1
    }

    // Second pass: compute split indices
    indices: int[count + 1]
    indices[0] = 0
    indices[count] = length(string)

    i = 0
    j: int = 1
    while i < length(string) {
        if string[i] == character {
            indices[j] = i
            j = j + 1
        }
        i = i + 1
    }

    // Third pass: compute splits
    splits: int[count][]

    i = 0

    while i + 1 < length(indices) {
        low: int = indices[i]
        high: int = indices[i + 1]

        // Skip split character for subsequent splits
        if i > 0 {
            low = low + 1
        }

        split': int[high - low]
        j = 0

        while j < length(split') {
            split'[j] = string[low + j]
            j = j + 1
        }

        splits[i] = split'
        i = i + 1
    }

    return splits
}
