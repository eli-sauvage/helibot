import { CategoryChannel, Client, Guild, VoiceChannel } from 'discord.js'
const mots = ["SAPRISTI SAUCISSE", "CHOQUET", "ATTISER", "MAGNETS", "PARAGES", "BRANDEE", "WHARTON", "SOUPCON", "OTERAIT", "POURREZ", "MADISON", "ROSAIRE", "PLONGEZ", "INFECTE", "STOPPEZ", "RICANER", "ENJAMBE", "CHARGER", "NYUNGWE", "UTERINE", "SOUDOYE", "DIOXYDE", "RESETTE", "GAUSSES", "SUSCITA", "REPERER", "EXERCEZ", "WYSIWYG", "PIGNOLE", "PHENOME", "DEPECER", "SALLIER", "REFEREZ", "PARTONS", "REMISEE", "FARNELL", "BAISAIT", "SEDUIRA", "FAUTIFS", "HELICES", "FORCENE", "LAVEURS", "CORDONS", "PLAIGNE", "CHAMANE", "BIGBANG", "DENEIGE", "ENCULEE", "CALINOU", "CHOQUET", "ATTISER", "MAGNETS", "PARAGES", "BRANDEE", "WHARTON", "SOUPCON", "OTERAIT", "POURREZ", "MADISON", "JUTLAND", "ROSAIRE", "PLONGEZ", "INFECTE", "STOPPEZ", "RICANER", "ENJAMBE", "CHARGER", "NYUNGWE", "BECAUSE", "UTERINE", "SOUDOYE", "DIOXYDE", "RESETTE", "GAUSSES", "SUSCITA", "REPERER", "EXERCEZ", "WYSIWYG", "PIGNOLE", "PHENOME", "DEPECER", "SALLIER", "REFEREZ", "PARTONS", "REMISEE", "FARNELL", "BAISAIT", "SEDUIRA", "FAUTIFS", "HELICES", "FORCENE", "LAVEURS", "CORDONS", "PLAIGNE", "CHAMANE", "BIGBANG", "DENEIGE", "ENCULEE", "CALINOU", "SAROUEL", "PALADIN", "ARTISTE", "VENDOME", "YAOURTS", "TOUTAFE", "RECEPER", "REMARIE", "JUGULER", "KHALIFA", "APORIES", "OCTROYE", "JETEURS", "MADONNE", "ACETATE", "ABOUTIS", "SOULANT", "BLEUTES", "ZAPPEES", "GREFFES", "ANNEXEE", "CLONALE", "HERAUTS", "DELICES", "BINOCLE", "CORDEAU", "COMEDIE", "ABONDER", "IGUANES", "PHASMES", "ABIMENT", "BOVIDES", "CAUDRON", "INCULPE", "CODEURS", "CHAMSIN", "SCHWARZ", "COULANT", "REDIFFS", "BOURRIN", "FRACTAL", "CHTITES", "ACHETER", "SEZANNE", "BIDASSE", "ARSENIC", "INTIMES", "GAUFRER", "VIVANTE", "RENDUES", "PINCAIT", "GILBERT", "OUKACHA", "FOUTAIT", "BRISERA", "CAHUTES", "IMPOSTE", "COUTENT", "SOMBREE", "RECORDA", "CONTENT", "CYCLONE", "AMERIKA", "MEUGLER", "CRACHES", "MAGHRIB", "NORFOLK", "CRETONS", "TANNEUR", "DIMITRI", "ABRUTIT", "NIQUAIT", "LEGALES", "SOUDAIT", "FASCINE", "BRITISH", "ESSAIMS", "ZEMMOUR", "CLEGUER", "COMBAVA", "BREVETE", "STABILO", "RAVAGEA", "FAUCHER", "MONTRER", "PENDONS", "PEELING", "FIGUIER", "BOUVIER", "LEURRES", "FLETRIE", "CONTRER", "CESSEES", "VERMONT", "LOUDEAC", "ETHANOL", "ESPERAI", "VENERER", "RAISINE", "ARRACHE", "TOUCHEE", "DEPECES", "CHECKEZ", "EMBALLA", "EMIGRER", "SOLIDES", "DRESSEZ", "GRANITE", "NARVALO", "SUCRENT", "BERMUDA", "CROIRAS", "TRUITES", "ACCISES", "LOUPEES", "FLOODEZ", "FINESSE", "PIGEAIT", "CELIBAT", "ETUDIES", "MENOTTE", "BANNIES", "MAJOREE", "ENTETEE", "FONCANT", "APAISES", "VANTANT", "PREVIEW", "ALARMEE", "GERSOIS", "FRISBEE", "LIERONT", "PERCERA", "FREEGAN", "FORBACH", "SPOILES", "LEOPARD", "ROMPENT", "ERUPTIF", "STIPULE", "CROPPEE", "OCCUPEZ", "MINAGES", "MARELLE", "MINOENS", "PAUMIEZ", "VITRAIL", "TACHEES", "REFUMER", "FISCALE", "CLEGUER", "COLONNA", "VALIDEZ", "ACCISES", "CYNIQUE", "FUMERAI", "RESINES", "TIGRONS", "PRIEURS", "ANSELME", "AMBIGUE", "BANCALS", "MANILLA", "MAURICE", "REVERER", "CONSENS", "LISSANT", "TESTEUR", "PACTISE", "VIBRERA", "POMEROL", "SENSURE", "VEROLER", "SUINTER", "GRISEES", "DEVOREE", "MILDIOU", "DEBATTU", "DECHUES", "WINSTON", "VENANTE", "ENFILEZ", "PISTENT", "AGRAVER", "ACIDULE", "AMORCER", "LUPANAR", "BIOFILM", "DEESSES", "DAMARIS", "HAMMAMS", "INFANTE", "DENTEES", "CAMPENT", "MAURICE", "DETENDS", "OCTAVES", "BADGENT", "VIGNALE", "PROLINE", "SPOLIER", "OUTDOOR", "MYKONOS", "AUBAINE", "MALPOLI", "TENTION", "MERLEAU", "HASHISH", "GEOLIER", "REGALER", "SCEPTRE", "CARACAL", "DURIANS", "OCCUPEZ", "VANESSA", "RELIEES", "FIANCEE", "POUVAIT", "DAMBACH", "BRISERA", "COUTUME", "EXPORTS", "DISCORD", "BOUILLE", "ACCORDA", "FORCENE", "FANTOME", "MACEREE", "AVENTIN", "CONFINS", "ARTICLE", "PREFETS", "VISIONS", "APPARTS", "RECYCLE", "JOUFFLU", "EXOGENE", "LARGUEE", "ENERVES", "CAUSAUX", "REGENCE", "GIVREES", "IGNAMES", "ROULERA", "ROUPIES", "CREOLES", "CLOITRE", "EVASIVE", "LIVRETS", "PAPYRUS", "NORMALE", "BARRAIS", "BATTENT", "ETENDRE", "DEMERDE", "INCLUSE", "FERMANT", "TOUSSES", "TAQUINS", "ENDENTE", "FOURNIL", "HANOVRE", "TRADUIS", "SYRPHES", "DECISIF", "BERTRIX", "PLAIRAS", "EXILANT", "ZELANDE", "FRONTEX", "ANCREES", "GRIGNON", "BOURRET", "BATIRAI", "DAMEUSE", "RECOURU", "AFFOLEE", "DYSPNEE", "HERSANT", "TOCARDS", "ANIMAUX", "AUXDITS", "EXIGENT", "SANTINI", "JOUEREZ", "FASCIES", "DESAXEE", "CORRECT", "ZONARDS", "CAUSAUX", "AFRIQUE", "KABBALE", "MAIZENA", "DEBACHE", "SIPHONE", "PESTAIT", "LEADAIS", "LAVABLE", "INSIGHT", "ENVOYER", "ISLANDE", "JACENTE", "PATCHES", "JETABLE", "MATHIAS", "HUMECTE", "SAURONT", "RAGOUTS", "BALISES", "PRONENT", "MEVENTE", "REJETER", "ENFOIRE", "TONTINE", "ANSELME", "PASSEUR", "JEANNOT", "SOMALIS", "HEGEMON", "COSSUES", "SULFURE", "ELISENT", "FERRAND", "BERNOIS", "POTAGES", "EREINTE", "REFUGES", "SIGNETS", "MEGALOS", "SOLVANT", "MIRAGES", "DUEMENT", "RABBINS", "BRULEUR", "MATTAIS", "INFESTE", "CLASSEE", "MANDELA", "ENTACHE", "PRISAIT", "FESTONS", "DESUNIS", "LAMBEAU", "TRIPLET", "NEWTONS", "POELEES", "BHOUTAN", "HAUTEUR", "SHOOTEZ", "PENSAIT", "STOCKEE", "DIKTATS", "FRANGIN", "MODALES", "AUBIERE", "CHANDON", "RACIALE", "DECOULE", "ARTISAN", "MIKHAIL", "ARYENNE", "NUIRAIT", "CORRENS", "CASTELA", "CISEAUX", "AGENDAS", "BALADOS", "SWANSEA", "MALACCA", "NIMBAIT", "PASSONS", "BUMPERS", "SURVECU", "ENTRONS", "CRANIEN", "ASSIEGE", "PRIMAUX", "ALBERIC", "SURFACE", "GLUANTS", "PAUSANT", "ALPINES", "EGALISE", "NAGEAIT", "AUSSIES", "NEWLOOK", "NOTOIRE", "CARREES", "REGALEE", "LAPIDES", "ENTENDE", "CANIGOU", "ENSABLE", "JUGERAI", "DETOURS", "MATERNE", "FOUILLE", "INSPIRA", "DEMARRA", "IMPACTE", "TRIPLEX", "BLOTTIE", "CHIBRES", "LAMARCK", "LEADAIS", "ABRITER", "PIONCER", "BERCENT", "TAGLINE", "FLEURER", "SOCIALE", "AEROGEL", "RUPTURE", "EVZONES", "INCUBES", "COCASSE", "EDMONDE", "RATUREE", "QUASARS", "ARCHIVE", "DEBATTU", "RESIDUS", "VEILLEE", "GENDRES", "LEGRAND", "SCORIES", "MUSCLEE", "ROCHEUX", "ENIGMES", "CREUSER", "VANTARD", "AISANCE", "BALISTE", "SAVARIN", "COURATE", "NURSERY", "PIONCER", "COURAIT", "NORMANT", "COURENT", "PETOCHE", "HURLENT", "PROTEES", "BINOMES", "DIVAGUE", "AIDERAS", "ARTEMIS", "PRESENT", "ASSOCES", "MARSYAS", "RATIONS", "LEJEUNE", "FACILES", "TABASSE", "CARIBOU", "HABITAI", "ZONARDS", "DECENCE", "ZOZIAUX", "MEANDRE", "GOMMEES", "PREFERA", "QUATUOR", "EPAULER", "IMPOSTE", "FLOREAL", "MEZIERE", "DISCALE", "EGAREES", "INGENUE", "PANSANT", "MONTAGE", "GAMBIEN", "LEZARDE", "DEMENEE", "DOUCEUR", "GALLIUM", "CLOPPER", "MEFIANT", "IRRITEE", "GRATTON", "JACENTE", "EVACUEE", "YAKUSAS", "PANURGE", "RETAMEE", "ENVAHIT", "RAJOUTE", "FOIRERA", "AVEYRON", "SUBISSE", "HUBLOTS", "ARPEGES", "AURIANE", "SNIFFES", "MULATRE", "BIPEDAL", "SNIPERS", "JUDAISE", "MOLINIE", "DOUCHES", "JALONNE", "VIVIFIE", "SIGNALE", "ECOPANT", "LEGANES", "LEVEQUE", "LICENCE", "TAISAIT", "EXPERTS", "GRAISSE", "PAYSANS", "THILLOT", "RAPPELS", "NOYADES", "JUMPERS", "DEFORCE", "TRASHER", "HYGIENE", "FAVORIS", "MOIRANS", "CROTTIN", "SAVATER", "SANGUIN", "GRILLET", "LATENCE", "TUERAIT", "ORVILLE", "ASSAGIE", "CROISER", "BRULAIT", "VARIOLE", "MOUVOIR", "AMADOUE", "ELUDENT", "MANIEES", "CASCADE", "GRADUEL", "CENDREE", "TOUCHES", "IXELLES", "PINCANT", "CRENELE", "BAROQUE", "ZEMMOUR", "BRUNIES", "VENTILE", "LOGEAIT", "FERRONS", "HONNIES", "BERLINE", "FEDERAL", "ARROSER", "RIGOTTE", "LISERON", "PERDRIX", "CUISTOS", "PASSAIT", "INCARNE", "PINTXOS", "DREYFUS", "POSTURE", "DEPHASE", "TANNENT", "KICKERS", "EDIFIER", "SPLINES", "NASEAUX", "SITTERS", "BEGAIES", "IMPUTES", "LESDITS", "PAGEANT", "LOGEANT", "NOTABLE", "FRAGGER", "BAYONNE", "ECHOUEZ", "BUKKAKE", "CATCHER", "TRADERS", "RAJOUTS", "FRITTEE", "CAILLOU", "POINDRE", "COTONOU", "DEROGER", "ETAPLES", "AGILITY", "LIVRENT", "CURETON", "SPINAUX", "POUTRES", "CONTANT", "STIPULE", "SUBSIDE", "DEVISES", "FALUCHE", "SUDISTE", "MENELAS", "ENDORME", "ACCROIT", "DUBSTEP", "OBUSIER", "ZAMBIEN", "DECENTE", "ESCIENT", "CHAOUIS", "FIASCOS", "WARBURG", "PARENTS", "WILAYAS", "KANTIEN", "MUTISME", "PUPITRE", "PRONERA", "ASIATES", "ETRIPER", "CALMAIS", "ESTIVES", "PORTERA", "TOCARDS", "MOCHICA", "FRAISES", "VILENIE", "EVASIVE", "PUTSCHS", "GERBAGE", "ORANGEE", "PROJETA", "STUPEUR", "CONSUMA", "PIOCHES", "DEROBEE", "VACARME", "TOURNAI", "ETAMPES", "TROMBES", "CANELES", "HERMANT", "TRITURE", "COUGUAR", "CUMULES", "PETOCHE", "MANIERE", "BIPEDAL", "SENIORS", "URINANT", "REVEUSE", "GEORGIA", "BLAMERA", "RETRECI", "MAUGREE", "TROQUET", "BURKINA", "ENTORSE", "DIGERER", "BOSNIEN", "CRUDITE", "MAQUENT", "FOURBIR", "SHIKOKU", "MINOENS", "LAURENS", "CABBALE", "PLAGIES", "EMBOITE", "BRITISH", "LEAKEES", "JOUIONS", "BANNENT", "MATANTE", "REVOLUS", "ANNEXEE", "DECEDES", "CRYPTEE", "TORONTO", "SIMPLON", "POUGNER", "CORRIGE", "ATTEINS", "SUMATRA", "PERREUX", "TOLARDS", "NICHEUR", "TRIPLET", "MUFFINS", "SEVERAC", "RISQUEZ", "FERMION", "FAVELAS", "EXECRES", "BROYANT", "MATCHES", "GOSPORT", "LOGUENT", "LANCEUR", "CLASSER", "PARVENU", "SENSUEL", "PARTONS", "DIZAINE", "REGLAIT", "GRIGRIS", "REDONDE", "PIROGUE", "CESSANT", "TORRIDE", "POSTIEZ", "MISSENT", "OXYGENE", "MALSAIN", "DETAXER", "LASURER", "CYMBALE", "DEVRONT", "REDUITE", "ENFOUIE", "MONTAGE", "TURKERS", "REMIXER", "AMAIGRI", "DESIREZ", "WAUTERS", "GLOBAUX", "COMIQUE", "REBAISE", "VIOLEUR", "POINTUS", "OMANAIS", "TRIBUTS", "NIAISES", "JUKEBOX", "SAUTEUR", "ECHAUDE", "SAPIENS", "GOULASH", "TALOCHE", "GUETTES", "DETENDE", "TRIBUTE", "ENZYMES", "ACTRICE", "BIGEARD", "ROUGIES", "ASSEOIR", "DEVERSA", "PREFERE", "ECROUES", "DISPUTA", "OUBLIEZ", "CANTINA", "ESCAPES", "CADRANS", "OBSEDEE", "PERCAGE", "PASSIVE", "BAGNEUX", "SIBERIE", "CAMPARI", "REPORTE", "VIANDOX", "FORMAIT", "LEGRAND", "SONINKE", "SURPRIS", "BITURER", "ZINGUES", "ABOLIRA", "REFUTER", "MOLINIE", "EVENTER", "PLURIEL", "BICHONS", "ARSENAL", "OPTERAI", "FORMENT", "DENTREE", "APITOIE", "SECOUES", "ROUSSIS", "CROUPES", "YANKEES", "LOUKOUM", "APACHES", "SERRANO", "AJOUTEE", "ENCREUR", "PISSENT", "SIKASSO", "QUILLES", "FOURREE", "LAINAGE", "OFFREUR", "CABLEES", "ORDINAL", "SERIEUX", "ZIEUTES", "SOMMITE", "SCHNECK", "XIIIEME", "EXAUCER", "MEUBLER", "NIBARDS", "PROUVER", "MATANTE", "DESDITS", "GRUGENT", "TEQUILA", "NIERONT", "DETENTE", "MISKINE", "BRESSON", "MOURREZ", "NETBOOK", "DECOTES", "LETTONE", "BARDENT", "REDIGER", "DUCRUET", "REVIENS", "LIGURIE", "ABAQUES", "PRIERES", "LECHENT", "KABBALE", "GREDINS", "FIBRENT", "FATBIKE", "SHEBABS", "BLUFFES", "CHOMAGE", "STRUDEL", "FENDANT", "POTERIE", "APPORTE"]

export class InitVocaux{
    guild: Guild
    parent: CategoryChannel
    general: VoiceChannel
    constructor(guild: Guild, parent:CategoryChannel, general:VoiceChannel){
        this.guild = guild
        this.parent = parent
        this.general = general
        this.testChannels()
    }
    getVoiceChannels = () => this.parent.children.filter(channel => {
        if (channel.type != "voice") return false
        if (channel.id == this.guild.afkChannelID) return false
        return true
    })
    testChannels(){
        let channs = this.getVoiceChannels()//.filter(chann => chann.id != general.id)//channs not general
        let numberOfEmpties = channs.filter(chann=>chann.members.size == 0).size//empty channs count
        channs.forEach(channel=>{
            if(numberOfEmpties == 1) return //ok
            else if (numberOfEmpties > 1 && channel.members.size == 0 && channel.id != this.general.id){
                channel.delete()
                numberOfEmpties --
            }else if(numberOfEmpties == 0){
                this.guild.channels.create(mots[Math.floor(Math.random() * mots.length)], {
                    type: "voice",
                    parent: this.parent.id
                }).then(channel => channel.setPosition(this.guild.channels.cache.filter(e => e.parentID == this.parent.id).size - 2))
                numberOfEmpties ++
            }
        })
    }
    // client.on("ready", () => {
    //     setInterval(() => {
    //         createIfFull()
    //         deleteIfEmpty()
    //     }, 30 * 1000)
    // })

    // client.on("voiceStateUpdate", (oldMemb, newMemb) => {
    //     createIfFull()
    //     deleteIfEmpty()
    // })
}
/*
    function createIfFull() {
        var full = true
        guild.channels.cache.filter(e =>
            e.type == "voice" &&
            e.id != guild.afkChannelID &&
            e.parent?e.parent.name == "Salons vocaux":false
        ).forEach(chann => {
            if (!chann.members.array().length)
                full = false
        })
        if (full) {
            guild.channels.create(mots[Math.floor(Math.random() * mots.length)], {
                type: "voice",
                parent: parent.id
            }).then(channel => channel.setPosition(guild.channels.cache.filter(e => e.parentID == parent.id).size - 2))
        }
    }
    function deleteIfEmpty() {
        let generalOccupé = general.members.array().length //nb gens ds general
        guild.channels.cache.filter(e =>
            e.type == "voice" &&
            e.id != general.id &&//general
            e.id != "535151379786760212" &&//afk
            e.name != "testHelibot" &&
            !e.members.array().length &&
            e.parent ? e.parent.name == "Salons vocaux" : false
        ).forEach(e => {
            if (!generalOccupé)
                e.delete()
            else
                generalOccupé = false//pr supprimer channels d'après (le general est tjr ocuppé tkt)
        })
    }
}




client.login(require("./../token"))
*/