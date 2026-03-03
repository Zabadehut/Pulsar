use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Beginner,
    Expert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricStatus {
    Implemented,
    Partial,
    Planned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVisibility {
    Visible,
    IndexedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fr,
    En,
}

impl Locale {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Self::En,
            _ => Self::Fr,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Fr => "fr",
            Self::En => "en",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Fr => Self::En,
            Self::En => Self::Fr,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceText {
    pub title: &'static str,
    pub summary: &'static str,
    pub beginner: &'static str,
    pub expert: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceEntry {
    pub id: &'static str,
    pub category: &'static str,
    pub panel: &'static str,
    pub status: MetricStatus,
    pub ui_visibility: UiVisibility,
    pub audience: Audience,
    pub aliases: &'static [&'static str],
    pub tags: &'static [&'static str],
    pub fr: ReferenceText,
    pub en: ReferenceText,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceEntryView {
    pub id: &'static str,
    pub category: &'static str,
    pub panel: &'static str,
    pub status: MetricStatus,
    pub ui_visibility: UiVisibility,
    pub audience: Audience,
    pub title: &'static str,
    pub summary: &'static str,
    pub beginner: &'static str,
    pub expert: &'static str,
    pub aliases: &'static [&'static str],
    pub tags: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub score: usize,
    pub entry: ReferenceEntryView,
}

const CATALOG: &[ReferenceEntry] = &[
    ReferenceEntry {
        id: "cpu.usage",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "cpu",
            "usage",
            "global cpu",
            "global cpu usage",
            "global cpu usage %",
            "processor",
        ],
        tags: &["cpu", "usage", "global", "processor"],
        fr: ReferenceText {
            title: "CPU global",
            summary: "Montre la charge CPU totale observee sur l'hote.",
            beginner: "Plus la valeur se rapproche de 100%, plus le processeur est occupe.",
            expert: "Le pourcentage agrege additionne user, nice, system, irq, softirq et steal selon la source OS.",
        },
        en: ReferenceText {
            title: "Global CPU",
            summary: "Shows total CPU load observed on the host.",
            beginner: "The closer the value is to 100%, the busier the processor is.",
            expert: "The aggregate percentage combines user, nice, system, irq, softirq and steal depending on OS source data.",
        },
    },
    ReferenceEntry {
        id: "cpu.load",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &["load", "load average", "1m", "5m", "15m"],
        tags: &["cpu", "load", "scheduler"],
        fr: ReferenceText {
            title: "Load average",
            summary: "Mesure la pression sur l'ordonnanceur sur 1, 5 et 15 minutes.",
            beginner: "Une load au-dessus du nombre de CPU peut signaler une file d'attente importante.",
            expert: "La semantique varie selon l'OS, mais reste utile comme signal de contention globale.",
        },
        en: ReferenceText {
            title: "Load average",
            summary: "Measures scheduler pressure over 1, 5 and 15 minutes.",
            beginner: "A load value above CPU count can indicate meaningful run queue pressure.",
            expert: "OS semantics differ, but it remains a useful host-level contention signal.",
        },
    },
    ReferenceEntry {
        id: "cpu.iowait",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["iowait", "io wait", "cpu wait"],
        tags: &["cpu", "disk", "latency", "linux"],
        fr: ReferenceText {
            title: "CPU iowait",
            summary: "Temps passe par le CPU a attendre des IO blocantes.",
            beginner: "Une hausse d'iowait peut pointer vers un stockage lent ou sature.",
            expert: "Sur Linux, cette mesure vient des compteurs CPU et doit etre lue avec la latence disque et la queue depth.",
        },
        en: ReferenceText {
            title: "CPU iowait",
            summary: "CPU time spent waiting on blocking IO.",
            beginner: "Rising iowait can indicate slow or saturated storage.",
            expert: "On Linux this comes from CPU accounting and should be read alongside disk latency and queue depth.",
        },
    },
    ReferenceEntry {
        id: "memory.pressure",
        category: "memory",
        panel: "memory",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "memory pressure",
            "memory pressure score",
            "pressure",
            "available memory",
        ],
        tags: &["memory", "pressure", "available", "swap"],
        fr: ReferenceText {
            title: "Pression memoire",
            summary: "Indice derive pour estimer la tension sur la memoire de l'hote.",
            beginner: "Une forte pression memoire signifie qu'il reste peu de marge avant swap ou reclaim agressif.",
            expert: "Pulsar derive ce score a partir de la memoire disponible, de l'usage et des compteurs associes.",
        },
        en: ReferenceText {
            title: "Memory pressure",
            summary: "Derived index estimating how stressed host memory is.",
            beginner: "High pressure means little margin remains before swap or aggressive reclaim.",
            expert: "Pulsar derives this score from available memory, usage and related counters.",
        },
    },
    ReferenceEntry {
        id: "memory.swap",
        category: "memory",
        panel: "memory",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "swap",
            "swap total used",
            "swap total / used",
            "paging",
            "swpin",
            "swpout",
        ],
        tags: &["memory", "swap", "paging"],
        fr: ReferenceText {
            title: "Swap",
            summary: "Montre l'utilisation de l'espace d'echange disque par la memoire virtuelle.",
            beginner: "Une forte activite swap peut ralentir fortement la machine.",
            expert: "Le swap doit etre interprete avec la pression memoire, pgin/pgout et les alertes.",
        },
        en: ReferenceText {
            title: "Swap",
            summary: "Shows disk-backed virtual memory usage.",
            beginner: "Heavy swap activity can slow the host down significantly.",
            expert: "Read swap together with memory pressure, pgin/pgout and alerts.",
        },
    },
    ReferenceEntry {
        id: "disk.await",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["await", "latency", "disk latency", "storage latency"],
        tags: &["disk", "await", "latency", "storage"],
        fr: ReferenceText {
            title: "Disk await",
            summary: "Latence moyenne observee par IO terminee.",
            beginner: "Plus cette valeur monte, plus les operations disque prennent du temps.",
            expert: "Pulsar la derive des compteurs de temps et d'IO completes, utile avec util% et queue depth.",
        },
        en: ReferenceText {
            title: "Disk await",
            summary: "Average latency observed per completed IO.",
            beginner: "Higher values mean disk operations take longer to finish.",
            expert: "Pulsar derives it from IO completion and timing counters; read it with util% and queue depth.",
        },
    },
    ReferenceEntry {
        id: "disk.queue_depth",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["queue depth", "qd", "io queue"],
        tags: &["disk", "queue", "latency", "saturation"],
        fr: ReferenceText {
            title: "Queue depth",
            summary: "Approximation du nombre moyen d'IO en attente ou en cours.",
            beginner: "Une queue depth qui monte avec la latence indique souvent une saturation.",
            expert: "Cette valeur vient du temps IO pondere, donc elle reste une approximation aggregate par device.",
        },
        en: ReferenceText {
            title: "Queue depth",
            summary: "Approximation of the average number of pending or active IOs.",
            beginner: "If queue depth rises with latency, storage saturation is likely.",
            expert: "This comes from weighted IO time, so it remains an aggregated per-device approximation.",
        },
    },
    ReferenceEntry {
        id: "network.tcp",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "tcp",
            "connections",
            "total tcp connections",
            "established tcp connections",
            "established",
            "listen",
            "time_wait",
        ],
        tags: &["network", "tcp", "connections", "listen"],
        fr: ReferenceText {
            title: "Connexions TCP",
            summary: "Resume l'etat courant des connexions reseau TCP.",
            beginner: "Established montre les connexions actives, Listen les sockets en attente, TimeWait les fins de session recentes.",
            expert: "Une hausse anormale de TimeWait, retrans ou syn peut signaler un probleme applicatif ou reseau.",
        },
        en: ReferenceText {
            title: "TCP connections",
            summary: "Summarizes the current state of TCP network connections.",
            beginner: "Established shows active sessions, Listen waiting sockets, TimeWait recent closed sessions.",
            expert: "Unusual rises in TimeWait, retrans or syn states can point to application or network issues.",
        },
    },
    ReferenceEntry {
        id: "network.retrans",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["retrans", "retransmits", "packet loss"],
        tags: &["network", "retrans", "loss", "tcp"],
        fr: ReferenceText {
            title: "Retransmissions",
            summary: "Compteur de segments TCP retransmis.",
            beginner: "Une hausse peut indiquer perte reseau, congestion ou cible lente.",
            expert: "A lire avec le debit, les erreurs, l'etat des connexions et la saturation applicative.",
        },
        en: ReferenceText {
            title: "Retransmissions",
            summary: "Counter of retransmitted TCP segments.",
            beginner: "Rising values can indicate packet loss, congestion or a slow peer.",
            expert: "Read together with throughput, errors, connection states and application saturation.",
        },
    },
    ReferenceEntry {
        id: "linux.psi",
        category: "linux",
        panel: "linux",
        status: MetricStatus::Partial,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["psi", "pressure stall", "stall", "linux pressure"],
        tags: &["linux", "psi", "pressure", "cpu", "memory", "io"],
        fr: ReferenceText {
            title: "PSI Linux",
            summary: "Pressure Stall Information mesure le temps perdu a cause de CPU, memoire ou IO.",
            beginner: "Si PSI monte, des taches restent bloquees faute de ressources.",
            expert: "Le avg10 est tres utile pour voir une degradation recente, surtout combine a cgroup et alerts.",
        },
        en: ReferenceText {
            title: "Linux PSI",
            summary: "Pressure Stall Information measures time lost to CPU, memory or IO pressure.",
            beginner: "When PSI rises, tasks are getting stalled by missing resources.",
            expert: "avg10 is especially useful for recent degradation, particularly with cgroup and alert context.",
        },
    },
    ReferenceEntry {
        id: "linux.cgroup",
        category: "linux",
        panel: "linux",
        status: MetricStatus::Partial,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &[
            "cgroup",
            "container",
            "containers cgroup v2",
            "containers / cgroup v2",
            "cpu throttle",
            "memory max",
        ],
        tags: &["linux", "cgroup", "container", "limits"],
        fr: ReferenceText {
            title: "Cgroup v2",
            summary: "Expose les limites et usages de ressources du groupe de controle courant.",
            beginner: "Pratique pour savoir si le processus tourne dans un conteneur ou sous quotas.",
            expert: "La memoire max, les pids et le throttling CPU aident a differencier un probleme host d'une limite imposee.",
        },
        en: ReferenceText {
            title: "Cgroup v2",
            summary: "Shows resource limits and usage for the current control group.",
            beginner: "Useful to tell whether the process runs inside a container or quota.",
            expert: "Memory max, pid limits and CPU throttling help separate host pressure from imposed limits.",
        },
    },
    ReferenceEntry {
        id: "process.cpu",
        category: "process",
        panel: "process",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &["process cpu", "top process", "pid", "threads"],
        tags: &["process", "cpu", "pid", "top"],
        fr: ReferenceText {
            title: "Top processus",
            summary: "Liste les processus les plus visibles selon CPU et autres compteurs.",
            beginner: "Commencez ici pour voir quel processus consomme CPU, memoire ou descripteurs.",
            expert: "La vue est utile pour un tri rapide, mais doit etre recroisee avec watch, snapshot ou replay.",
        },
        en: ReferenceText {
            title: "Top processes",
            summary: "Lists the most visible processes by CPU and related counters.",
            beginner: "Start here to see which process is using CPU, memory or file descriptors.",
            expert: "This is a fast triage view and should be cross-checked with watch, snapshot or replay.",
        },
    },
    ReferenceEntry {
        id: "process.jvm",
        category: "process",
        panel: "process",
        status: MetricStatus::Partial,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["jvm", "java", "jvm detection"],
        tags: &["process", "jvm", "java"],
        fr: ReferenceText {
            title: "Detection JVM",
            summary: "Marque certains processus comme JVM selon des heuristiques simples.",
            beginner: "Le tag JVM aide a reperer rapidement une application Java dans la liste.",
            expert: "Ce n'est pas encore une detection profonde de runtime; le signal reste heuristique.",
        },
        en: ReferenceText {
            title: "JVM detection",
            summary: "Marks some processes as JVMs using simple heuristics.",
            beginner: "The JVM tag helps spot Java applications quickly in the process list.",
            expert: "This is not deep runtime detection yet; the signal is still heuristic.",
        },
    },
    ReferenceEntry {
        id: "alerts",
        category: "derived",
        panel: "alerts",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &["alerts", "threshold alerts", "warning", "critical", "health"],
        tags: &["alerts", "health", "thresholds"],
        fr: ReferenceText {
            title: "Alertes",
            summary: "Les alertes synthetisent les signaux les plus urgents du snapshot.",
            beginner: "Utilisez-les comme point d'entree, puis remontez vers CPU, memoire, disque ou reseau.",
            expert: "Les alertes actuelles sont locales et basees sur seuils; elles donnent du contexte mais pas une RCA complete.",
        },
        en: ReferenceText {
            title: "Alerts",
            summary: "Alerts summarize the most urgent signals in the current snapshot.",
            beginner: "Use them as an entry point, then drill into CPU, memory, disk or network.",
            expert: "Current alerts are local and threshold-based; they provide context, not full RCA.",
        },
    },
    ReferenceEntry {
        id: "expert.pressure_view",
        category: "derived",
        panel: "linux",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "pressure+",
            "expert pressure",
            "pressure view",
            "diagnostic pression",
        ],
        tags: &["expert", "pressure", "psi", "cgroup", "memory"],
        fr: ReferenceText {
            title: "Vue experte pression",
            summary: "Regroupe pression memoire, PSI, throttling et processus bloques.",
            beginner: "Vue reservee au diagnostic avance de contention locale.",
            expert: "Doit aider a separer pression host, pression cgroup et symptomes process.",
        },
        en: ReferenceText {
            title: "Expert pressure view",
            summary: "Groups memory pressure, PSI, throttling and stalled processes.",
            beginner: "This view is meant for advanced local contention diagnosis.",
            expert: "It should separate host pressure, cgroup pressure and process symptoms.",
        },
    },
    ReferenceEntry {
        id: "expert.network_view",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "network+",
            "expert network",
            "network diagnosis",
            "socket diagnosis",
        ],
        tags: &["expert", "network", "tcp", "udp", "retrans", "socket"],
        fr: ReferenceText {
            title: "Vue experte reseau",
            summary: "Met en avant debit, pps, retransmissions et etats TCP.",
            beginner: "Plus technique qu'une simple vue trafic, utile pour incidents reseau.",
            expert: "Pensee comme une lecture locale inspiree des reflexes Wireshark sans capture brute.",
        },
        en: ReferenceText {
            title: "Expert network view",
            summary: "Highlights throughput, pps, retransmissions and TCP states.",
            beginner: "More technical than a simple traffic view and useful during network incidents.",
            expert: "Designed as a local readout inspired by Wireshark instincts without raw packet capture.",
        },
    },
    ReferenceEntry {
        id: "expert.jvm_view",
        category: "process",
        panel: "process",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "jvm+",
            "expert jvm",
            "jvm diagnosis",
            "java diagnosis",
        ],
        tags: &["expert", "jvm", "java", "threads", "runtime"],
        fr: ReferenceText {
            title: "Vue experte JVM",
            summary: "Met en avant CPU, memoire, threads, FDs et IO des JVM detectees.",
            beginner: "Donne un premier tri local avant des outils JVM plus intrusifs.",
            expert: "Ce n'est pas encore un thread dump analyzer, mais une passerelle rapide vers la bonne JVM.",
        },
        en: ReferenceText {
            title: "Expert JVM view",
            summary: "Highlights CPU, memory, threads, FDs and IO for detected JVMs.",
            beginner: "Provides a first local triage before using more intrusive JVM tooling.",
            expert: "This is not yet a thread-dump analyzer, but it is a fast bridge to the right JVM.",
        },
    },
    ReferenceEntry {
        id: "expert.disk_pressure_view",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "disk+",
            "expert disk",
            "disk pressure",
            "disk contention",
        ],
        tags: &["expert", "disk", "await", "queue", "latency", "disk sleep"],
        fr: ReferenceText {
            title: "Vue experte contention disque",
            summary: "Croise disque chaud, latence, queue depth et processus en sommeil disque.",
            beginner: "Utile pour repondre vite a la question : quel disque souffre et qui attend ?",
            expert: "Doit aider a relier saturation bloc, service time et symptomes process.",
        },
        en: ReferenceText {
            title: "Expert disk contention view",
            summary: "Cross-checks hot disks, latency, queue depth and disk-sleeping processes.",
            beginner: "Useful to answer quickly: which disk is hurting and who is waiting?",
            expert: "It should connect block saturation, service time and process symptoms.",
        },
    },
    ReferenceEntry {
        id: "expert.pressure_paths",
        category: "derived",
        panel: "linux",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "pressure paths",
            "host mem pressure",
            "psi mem avg10",
            "cpu throttled",
        ],
        tags: &["expert", "pressure", "psi", "cgroup", "drilldown"],
        fr: ReferenceText {
            title: "Chemins de pression",
            summary: "Table de drill-down qui separe pression hote, PSI, load et cgroup.",
            beginner: "Utile si vous voulez comprendre d'ou vient la tension avant d'ouvrir d'autres outils.",
            expert: "Le but est de distinguer pression host-wide, limite cgroup et symptomes CPU/IO dans une seule vue.",
        },
        en: ReferenceText {
            title: "Pressure paths",
            summary: "Drill-down table separating host pressure, PSI, load and cgroup signals.",
            beginner: "Useful when you want to understand where the stress comes from before opening heavier tools.",
            expert: "The goal is to separate host-wide pressure, cgroup limits and CPU/IO symptoms in one view.",
        },
    },
    ReferenceEntry {
        id: "expert.pressure_lenses",
        category: "derived",
        panel: "linux",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "pressure lenses",
            "reclaim score",
            "swap push",
            "host cgroup gap",
        ],
        tags: &["expert", "pressure", "reclaim", "swap", "cgroup", "lenses"],
        fr: ReferenceText {
            title: "Lentilles de pression",
            summary: "Sous-vue pression qui condense reclaim, swap, ecart host/cgroup et stalls.",
            beginner: "Aide a voir si la tension vient surtout du reclaim, du swap ou d'une limite locale.",
            expert: "Utile pour separer manque de RAM, stalls IO et quota cgroup sans quitter la TUI.",
        },
        en: ReferenceText {
            title: "Pressure lenses",
            summary: "Pressure subview condensing reclaim, swap, host/cgroup gap and stalls.",
            beginner: "Helps show whether the stress is mostly reclaim, swap or a local limit issue.",
            expert: "Useful to separate RAM shortage, IO stalls and cgroup limits without leaving the TUI.",
        },
    },
    ReferenceEntry {
        id: "expert.socket_states",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "socket states",
            "tcp states",
            "syn sent recv",
            "close wait",
        ],
        tags: &["expert", "network", "socket", "tcp", "drilldown"],
        fr: ReferenceText {
            title: "Etats socket/TCP",
            summary: "Drill-down des etats TCP et UDP pour l'analyse incidente locale.",
            beginner: "Permet de voir si le probleme ressemble plus a un souci de lien ou de sessions.",
            expert: "A lire avec retransmissions, drops et debit d'interface pour approcher une lecture type Wireshark locale.",
        },
        en: ReferenceText {
            title: "Socket/TCP states",
            summary: "Drill-down of TCP and UDP states for local incident analysis.",
            beginner: "Helps tell whether the issue looks more like a link problem or a session problem.",
            expert: "Read together with retransmissions, drops and interface throughput for a local Wireshark-like readout.",
        },
    },
    ReferenceEntry {
        id: "expert.network_lenses",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "session lenses",
            "handshake pressure",
            "closing backlog",
            "loss path score",
        ],
        tags: &["expert", "network", "sessions", "handshake", "closing", "loss"],
        fr: ReferenceText {
            title: "Lentilles de session reseau",
            summary: "Sous-vue reseau qui separe pression handshake, backlog de fermeture et chemin de perte.",
            beginner: "Aide a lire rapidement si le probleme vient plus des sessions que du debit brut.",
            expert: "Condense plusieurs etats TCP/UDP en quelques lectures actionnables pour le triage local.",
        },
        en: ReferenceText {
            title: "Network session lenses",
            summary: "Network subview separating handshake pressure, closing backlog and loss path.",
            beginner: "Helps read quickly whether the issue is more session-driven than throughput-driven.",
            expert: "Condenses multiple TCP/UDP states into a few actionable local triage signals.",
        },
    },
    ReferenceEntry {
        id: "expert.jvm_hotspots",
        category: "process",
        panel: "process",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "jvm hotspots",
            "thread runtime",
            "thread analyzer",
            "runtime focus",
        ],
        tags: &["expert", "jvm", "threads", "runtime", "drilldown"],
        fr: ReferenceText {
            title: "Hotspots JVM",
            summary: "Table de drill-down JVM centree sur CPU, RSS, threads, FDs et IO.",
            beginner: "Utile pour identifier rapidement quelle JVM merite un outil plus intrusif.",
            expert: "Ce n'est pas encore un dump de threads, mais deja une orientation runtime locale tres rapide.",
        },
        en: ReferenceText {
            title: "JVM hotspots",
            summary: "JVM drill-down table focused on CPU, RSS, threads, FDs and IO.",
            beginner: "Useful to quickly identify which JVM deserves deeper tooling.",
            expert: "This is not yet a full thread dump, but it is already a fast local runtime orientation.",
        },
    },
    ReferenceEntry {
        id: "expert.jvm_profiles",
        category: "process",
        panel: "process",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "jvm profiles",
            "runtime profiles",
            "heap hint",
            "dominant pressure",
        ],
        tags: &["expert", "jvm", "runtime", "heap", "threads", "profiles"],
        fr: ReferenceText {
            title: "Profils runtime JVM",
            summary: "Sous-vue JVM qui classe role, pression dominante et hints de heap.",
            beginner: "Utile pour savoir vite quelle JVM regarder et pourquoi.",
            expert: "Cette lecture reste heuristique mais accelere le passage vers les outils JVM plus profonds.",
        },
        en: ReferenceText {
            title: "JVM runtime profiles",
            summary: "JVM subview classifying role, dominant pressure and heap hints.",
            beginner: "Useful to see quickly which JVM deserves attention and why.",
            expert: "This readout stays heuristic but speeds up the handoff to deeper JVM tools.",
        },
    },
    ReferenceEntry {
        id: "expert.disk_waiters",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "disk waiters",
            "waiters io",
            "processus d",
            "io waiters",
        ],
        tags: &["expert", "disk", "waiters", "io", "drilldown"],
        fr: ReferenceText {
            title: "Attenteurs disque / IO",
            summary: "Relie le disque chaud aux processus qui lisent, ecrivent ou attendent le blocage.",
            beginner: "Repond vite a la question : qui souffre du stockage maintenant ?",
            expert: "Pensee pour relier device, queue depth, await et symptomes process dans le meme ecran.",
        },
        en: ReferenceText {
            title: "Disk / IO waiters",
            summary: "Links the hot disk to the processes reading, writing or waiting on storage.",
            beginner: "Quickly answers: who is suffering from storage right now?",
            expert: "Designed to connect device, queue depth, await and process symptoms in the same screen.",
        },
    },
    ReferenceEntry {
        id: "expert.disk_lenses",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "disk lenses",
            "busy path",
            "latency path",
            "waiter pressure",
        ],
        tags: &["expert", "disk", "util", "await", "queue", "waiters", "lenses"],
        fr: ReferenceText {
            title: "Lentilles de contention disque",
            summary: "Sous-vue disque qui separe chemin charge, chemin latence, file et pression des attenteurs.",
            beginner: "Permet de voir vite si le probleme est plutot debit, latence ou attente processus.",
            expert: "Condense les angles util%, await, queue depth et attenteurs process dans une lecture courte.",
        },
        en: ReferenceText {
            title: "Disk contention lenses",
            summary: "Disk subview separating busy path, latency path, queue path and waiter pressure.",
            beginner: "Lets you see quickly whether the issue is more throughput, latency or process waiting.",
            expert: "Condenses util%, await, queue depth and waiting-process angles into one short readout.",
        },
    },
    ReferenceEntry {
        id: "cpu.per_core",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Partial,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &["per core", "core cpu", "cpu core", "per-core cpu usage"],
        tags: &["cpu", "core", "hotspot"],
        fr: ReferenceText {
            title: "CPU par coeur",
            summary: "Detail CPU coeur par coeur.",
            beginner: "Permet de voir un coeur saturer meme si le global reste modere.",
            expert: "Utile pour les workloads single-thread et les problemes d'affinite.",
        },
        en: ReferenceText {
            title: "Per-core CPU",
            summary: "CPU broken down per core.",
            beginner: "Lets you see one hot core even if total CPU remains moderate.",
            expert: "Useful for single-thread workloads and affinity issues.",
        },
    },
    ReferenceEntry {
        id: "cpu.scheduler",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "context switches",
            "interrupt count",
            "interrupts",
            "ctx",
            "irq count",
        ],
        tags: &["cpu", "scheduler", "interrupts"],
        fr: ReferenceText {
            title: "Signaux ordonnanceur CPU",
            summary: "Regroupe context switches et interruptions.",
            beginner: "Aide a reperer une agitation systeme inhabituelle.",
            expert: "A relier a load, irq%, softirq% et au profil de charge.",
        },
        en: ReferenceText {
            title: "CPU scheduler signals",
            summary: "Groups context switches and interrupts.",
            beginner: "Helps spot unusual system churn.",
            expert: "Read it with load, irq%, softirq% and workload profile.",
        },
    },
    ReferenceEntry {
        id: "cpu.steal",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Partial,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &["steal", "steal %", "stolen cpu"],
        tags: &["cpu", "steal", "virtualization"],
        fr: ReferenceText {
            title: "CPU steal",
            summary: "Temps CPU vole par la couche de virtualisation.",
            beginner: "Surtout utile en VM pour voir si l'hote manque de CPU reel.",
            expert: "Signal fort sous Linux, plus approximatif hors Linux.",
        },
        en: ReferenceText {
            title: "CPU steal",
            summary: "CPU time taken away by the virtualization layer.",
            beginner: "Mostly useful on VMs to see whether the host lacks real CPU time.",
            expert: "Strong signal on Linux, more approximate outside Linux.",
        },
    },
    ReferenceEntry {
        id: "cpu.pressure",
        category: "cpu",
        panel: "cpu",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "cpu pressure",
            "cpu pressure indicators",
            "scheduler pressure",
        ],
        tags: &["cpu", "pressure", "planned"],
        fr: ReferenceText {
            title: "Pression CPU",
            summary: "Indicateur de contention CPU plus direct que l'usage pur.",
            beginner: "Objectif : montrer quand les taches attendent vraiment du CPU.",
            expert: "Planifie pour completer usage, load et PSI.",
        },
        en: ReferenceText {
            title: "CPU pressure",
            summary: "A CPU contention signal more direct than raw usage.",
            beginner: "Goal: show when tasks are truly waiting for CPU time.",
            expert: "Planned to complement usage, load and PSI.",
        },
    },
    ReferenceEntry {
        id: "memory.breakdown",
        category: "memory",
        panel: "memory",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &[
            "total memory",
            "used memory",
            "free memory",
            "available memory",
            "cached memory",
        ],
        tags: &["memory", "used", "free", "available", "cached"],
        fr: ReferenceText {
            title: "Detail memoire",
            summary: "Total, utilise, libre, disponible, cache.",
            beginner: "La memoire disponible est souvent plus utile que la memoire libre brute.",
            expert: "La parite cache/buffers reste variable selon l'OS.",
        },
        en: ReferenceText {
            title: "Memory breakdown",
            summary: "Total, used, free, available, cached.",
            beginner: "Available memory is often more useful than raw free memory.",
            expert: "Cache/buffer parity still varies by OS.",
        },
    },
    ReferenceEntry {
        id: "memory.vm_counters",
        category: "memory",
        panel: "memory",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &["buffers", "dirty pages", "pgfault", "pgscan", "pgsteal"],
        tags: &["memory", "vm", "buffers", "dirty", "paging"],
        fr: ReferenceText {
            title: "Compteurs VM",
            summary: "Compteurs paging, reclaim et activite memoire.",
            beginner: "Plus utile pour du diagnostic avance que pour une lecture simple.",
            expert: "Permet de separer cache sain, manque de RAM et reclaim agressif.",
        },
        en: ReferenceText {
            title: "VM counters",
            summary: "Paging, reclaim and memory-activity counters.",
            beginner: "More useful for advanced diagnosis than simple daily reading.",
            expert: "Helps separate healthy cache, RAM shortage and aggressive reclaim.",
        },
    },
    ReferenceEntry {
        id: "disk.capacity",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &[
            "disk usage",
            "disk usage %",
            "disk free",
            "disk total used free",
            "disk total / used / free",
            "capacity",
            "filesystem usage",
            "per-filesystem detail parity",
        ],
        tags: &["disk", "filesystem", "capacity", "usage"],
        fr: ReferenceText {
            title: "Capacite disque",
            summary: "Total, libre, utilise et taux d'occupation.",
            beginner: "Un disque plein peut degrader l'hote et casser des applis.",
            expert: "La profondeur filesystem detaillee complete reste encore planifiee.",
        },
        en: ReferenceText {
            title: "Disk capacity",
            summary: "Total, free, used space and usage rate.",
            beginner: "A full disk can degrade the host and break applications.",
            expert: "Full filesystem-detail parity is still planned.",
        },
    },
    ReferenceEntry {
        id: "disk.performance",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "iops",
            "read iops",
            "write iops",
            "throughput",
            "read throughput",
            "write throughput",
            "disk utilization",
            "disk utilization %",
            "disk await latency",
            "disk await / latency",
            "util",
            "service time",
        ],
        tags: &["disk", "iops", "throughput", "utilization"],
        fr: ReferenceText {
            title: "Performance disque",
            summary: "IOPS, debit, util% et temps de service.",
            beginner: "Un haut debit seul n'est pas un probleme s'il n'y a pas de latence.",
            expert: "La saturation se lit avec await, queue depth, util% et debit ensemble.",
        },
        en: ReferenceText {
            title: "Disk performance",
            summary: "IOPS, throughput, util% and service time.",
            beginner: "High throughput alone is not a problem if latency stays low.",
            expert: "Saturation must be read through await, queue depth, util% and throughput together.",
        },
    },
    ReferenceEntry {
        id: "disk.structure",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "disk structure",
            "structure",
            "namespace",
            "mapper",
            "logical volume",
        ],
        tags: &["disk", "structure", "layout", "volume"],
        fr: ReferenceText {
            title: "Structure disque",
            summary: "Indique comment Pulsar classe le disque: bloc, namespace, mapper, volume logique ou autre.",
            beginner: "Aide a savoir si vous regardez un disque physique, une couche logique ou un volume OS.",
            expert: "Cette lecture reste heuristique selon l'OS, mais clarifie tres vite la nature du chemin de stockage.",
        },
        en: ReferenceText {
            title: "Disk structure",
            summary: "Shows how Pulsar classifies the disk: block, namespace, mapper, logical volume or similar.",
            beginner: "Helps tell whether you are looking at a physical disk, a logical layer or an OS volume.",
            expert: "This stays heuristic per OS, but it quickly clarifies the nature of the storage path.",
        },
    },
    ReferenceEntry {
        id: "disk.protocol",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &[
            "disk protocol",
            "proto",
            "nvme",
            "virtio",
            "scsi",
            "sata",
            "windows volume",
        ],
        tags: &["disk", "protocol", "nvme", "virtio", "scsi", "sata"],
        fr: ReferenceText {
            title: "Protocole disque",
            summary: "Hint de protocole ou de couche de transport: NVMe, virtio, SCSI/SATA, mapper, volume Windows.",
            beginner: "Utile pour savoir si le stockage suit un chemin local, virtuel ou logique.",
            expert: "Le signal est derive heuristiquement pour rester portable entre Linux, macOS et Windows.",
        },
        en: ReferenceText {
            title: "Disk protocol",
            summary: "Hint about protocol or transport layer: NVMe, virtio, SCSI/SATA, mapper, Windows volume.",
            beginner: "Useful to tell whether storage follows a local, virtual or logical path.",
            expert: "This signal is derived heuristically to stay portable across Linux, macOS and Windows.",
        },
    },
    ReferenceEntry {
        id: "disk.media",
        category: "disk",
        panel: "disk",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &["disk media", "media", "ssd", "virtual media", "hdd"],
        tags: &["disk", "media", "ssd", "virtual"],
        fr: ReferenceText {
            title: "Media disque",
            summary: "Hint de media: SSD, disque generique, virtuel, APFS ou inconnu selon l'OS.",
            beginner: "Aide a mieux lire les perf si la latence vient d'un SSD local ou d'une couche virtuelle.",
            expert: "Ce hint n'est pas une verite absolue mais une aide de lecture rapide portable.",
        },
        en: ReferenceText {
            title: "Disk media",
            summary: "Media hint: SSD, generic disk, virtual, APFS or unknown depending on OS.",
            beginner: "Helps read performance better depending on whether latency comes from local SSD or a virtual layer.",
            expert: "This hint is not absolute truth, but a fast portable aid for interpretation.",
        },
    },
    ReferenceEntry {
        id: "network.throughput",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &[
            "rx bytes",
            "rx bytes/sec",
            "tx bytes",
            "tx bytes/sec",
            "network throughput",
            "bandwidth",
        ],
        tags: &["network", "throughput", "rx", "tx"],
        fr: ReferenceText {
            title: "Debit reseau",
            summary: "Debit entrant et sortant par interface.",
            beginner: "Premier signal pour voir si une interface travaille beaucoup.",
            expert: "A croiser avec paquets, erreurs, drops et retrans.",
        },
        en: ReferenceText {
            title: "Network throughput",
            summary: "Inbound and outbound throughput per interface.",
            beginner: "First signal to see whether an interface is busy.",
            expert: "Cross-check it with packets, errors, drops and retrans.",
        },
    },
    ReferenceEntry {
        id: "network.structure",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "network structure",
            "topology",
            "interface topology",
            "bridge virtual",
            "host interface",
        ],
        tags: &["network", "topology", "bridge", "loopback", "vpn", "tunnel"],
        fr: ReferenceText {
            title: "Structure reseau",
            summary: "Classe l'interface comme loopback, pont virtuel, tunnel, agregat ou interface hote.",
            beginner: "Aide a savoir si vous lisez un lien physique, un chemin conteneur, un tunnel ou une interface locale.",
            expert: "Le signal est derive heuristiquement pour rester portable entre OS sans exiger d'APIs specifiques.",
        },
        en: ReferenceText {
            title: "Network structure",
            summary: "Classifies the interface as loopback, virtual bridge, tunnel, aggregate or host interface.",
            beginner: "Helps tell whether you are reading a physical link, container path, tunnel or local interface.",
            expert: "The signal is derived heuristically to stay portable across OSes without requiring platform-specific APIs.",
        },
    },
    ReferenceEntry {
        id: "network.family",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Beginner,
        aliases: &[
            "network family",
            "family",
            "wireless",
            "ethernet",
            "container family",
            "overlay",
        ],
        tags: &["network", "family", "ethernet", "wireless", "container", "overlay"],
        fr: ReferenceText {
            title: "Famille reseau",
            summary: "Hint de famille d'interface: ethernet, wireless, container, overlay ou local.",
            beginner: "Permet de replacer rapidement l'interface dans le bon contexte d'usage.",
            expert: "Ce hint aide a ne pas melanger trafic host, trafic conteneur et trafic overlay dans la meme lecture.",
        },
        en: ReferenceText {
            title: "Network family",
            summary: "Interface family hint: ethernet, wireless, container, overlay or local.",
            beginner: "Lets you quickly place the interface in the right usage context.",
            expert: "This hint helps avoid mixing host, container and overlay traffic in the same reading.",
        },
    },
    ReferenceEntry {
        id: "network.medium",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::Visible,
        audience: Audience::Expert,
        aliases: &["network medium", "media", "wired", "wifi", "virtual medium"],
        tags: &["network", "medium", "wired", "wifi", "virtual"],
        fr: ReferenceText {
            title: "Media reseau",
            summary: "Hint de media: cable, wifi, virtuel, overlay ou logiciel.",
            beginner: "Pratique pour lire plus vite si un souci vient d'un lien physique ou d'une couche logique.",
            expert: "Hint portable et heuristique, utile dans les vues locales, l'API et les exports.",
        },
        en: ReferenceText {
            title: "Network medium",
            summary: "Medium hint: wired, wifi, virtual, overlay or software.",
            beginner: "Useful to see faster whether an issue is tied to a physical link or a logical layer.",
            expert: "Portable heuristic hint used in local views, the API and structured exports.",
        },
    },
    ReferenceEntry {
        id: "network.packets",
        category: "network",
        panel: "network",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "packet rate",
            "rx packets",
            "rx packets/sec",
            "tx packets",
            "tx packets/sec",
            "pps",
        ],
        tags: &["network", "packets", "pps"],
        fr: ReferenceText {
            title: "Paquets reseau",
            summary: "Paquets recus et emis par seconde.",
            beginner: "Utile quand le debit semble faible mais qu'il y a beaucoup de petits paquets.",
            expert: "Aide a distinguer gros flux et trafic a forte frequence de paquets.",
        },
        en: ReferenceText {
            title: "Network packets",
            summary: "Packets received and sent per second.",
            beginner: "Useful when throughput looks low but lots of tiny packets are moving.",
            expert: "Helps separate large flows from packet-rate-heavy traffic.",
        },
    },
    ReferenceEntry {
        id: "network.errors_drops",
        category: "network",
        panel: "network",
        status: MetricStatus::Partial,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "errors",
            "rx tx errors",
            "rx/tx errors",
            "drops",
            "rx tx drops",
            "rx/tx drops",
            "packet drops",
        ],
        tags: &["network", "errors", "drops"],
        fr: ReferenceText {
            title: "Erreurs et drops reseau",
            summary: "Compteurs de paquets en erreur ou abandonnes.",
            beginner: "Une hausse persistante est un mauvais signal pour le chemin reseau.",
            expert: "La parite exacte varie encore selon l'OS, mais le signal reste critique.",
        },
        en: ReferenceText {
            title: "Network errors and drops",
            summary: "Counters of errored or dropped packets.",
            beginner: "A persistent rise is a bad sign for the network path.",
            expert: "Exact parity still varies by OS, but the signal remains critical.",
        },
    },
    ReferenceEntry {
        id: "network.udp_depth",
        category: "network",
        panel: "network",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "udp",
            "socket family",
            "udp depth",
            "udp socket family depth",
            "udp / socket family depth",
        ],
        tags: &["network", "udp", "planned"],
        fr: ReferenceText {
            title: "Profondeur UDP/socket",
            summary: "Vue reseau plus fine par famille de sockets, encore planifiee.",
            beginner: "Pas vitale au quotidien, utile pour cas reseau plus avances.",
            expert: "Manque encore pour une lecture fine des charges non-TCP.",
        },
        en: ReferenceText {
            title: "UDP/socket depth",
            summary: "Finer network view by socket family, still planned.",
            beginner: "Not vital daily, useful for more advanced network cases.",
            expert: "Still missing for precise reading of non-TCP workloads.",
        },
    },
    ReferenceEntry {
        id: "process.inventory",
        category: "process",
        panel: "process",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &[
            "top n",
            "top n process listing",
            "process rss",
            "process vsz",
            "process thread count",
            "thread count",
            "process fd count",
            "fd count",
            "process owner",
            "owner",
            "process read write bytes",
            "process read/write bytes",
            "process io",
            "basic jvm detection",
        ],
        tags: &["process", "rss", "vsz", "threads", "fd", "owner", "io"],
        fr: ReferenceText {
            title: "Inventaire processus",
            summary: "Regroupe RSS, VSZ, threads, FDs, owner et IO.",
            beginner: "Le point de depart pour voir quel processus pese vraiment sur l'hote.",
            expert: "Vue de tri rapide a completer par watch, snapshot et replay.",
        },
        en: ReferenceText {
            title: "Process inventory",
            summary: "Groups RSS, VSZ, threads, FDs, owner and IO.",
            beginner: "The starting point to see which process really weighs on the host.",
            expert: "Fast triage view to complement with watch, snapshot and replay.",
        },
    },
    ReferenceEntry {
        id: "process.jvm.deep",
        category: "process",
        panel: "process",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &["strong jvm awareness", "deep jvm", "jvm runtime"],
        tags: &["process", "jvm", "planned", "runtime"],
        fr: ReferenceText {
            title: "JVM approfondie",
            summary: "Visibilite JVM plus riche, encore planifiee.",
            beginner: "Objectif : faire mieux qu'un simple tag JVM.",
            expert: "Doit etendre l'heuristique actuelle avec de vrais signaux runtime.",
        },
        en: ReferenceText {
            title: "Deep JVM awareness",
            summary: "Richer JVM visibility, still planned.",
            beginner: "Goal: go beyond a simple JVM tag.",
            expert: "Should extend the current heuristic with real runtime signals.",
        },
    },
    ReferenceEntry {
        id: "process.thread_analysis",
        category: "process",
        panel: "process",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &["deep per-thread analysis", "thread analysis"],
        tags: &["process", "threads", "planned"],
        fr: ReferenceText {
            title: "Analyse par thread",
            summary: "Diagnostic detaille au niveau thread, encore planifie.",
            beginner: "Pas necessaire pour l'usage de base, mais utile pour blocages complexes.",
            expert: "Important pour hot threads, affinite CPU et internals runtime.",
        },
        en: ReferenceText {
            title: "Per-thread analysis",
            summary: "Detailed thread-level diagnosis, still planned.",
            beginner: "Not needed for basic use, but useful for complex stalls.",
            expert: "Important for hot threads, CPU affinity and runtime internals.",
        },
    },
    ReferenceEntry {
        id: "process.runtime_awareness",
        category: "process",
        panel: "process",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "python awareness",
            "runtime awareness",
            "python app-runtime awareness",
            "python / app-runtime awareness",
        ],
        tags: &["process", "runtime", "python", "planned"],
        fr: ReferenceText {
            title: "Awareness runtime",
            summary: "Visibilite applicative plus riche pour Python et autres runtimes.",
            beginner: "Objectif : mieux relier un processus a son comportement applicatif.",
            expert: "Etend Pulsar au-dela de la simple observabilite host.",
        },
        en: ReferenceText {
            title: "Runtime awareness",
            summary: "Richer application visibility for Python and other runtimes.",
            beginner: "Goal: connect a process more clearly to real application behavior.",
            expert: "Extends Pulsar beyond simple host observability.",
        },
    },
    ReferenceEntry {
        id: "system.identity",
        category: "system",
        panel: "cpu",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &[
            "hostname",
            "os version",
            "os name version",
            "os name / version",
            "kernel",
            "kernel version",
            "uptime",
            "architecture",
            "cpu count",
        ],
        tags: &["system", "metadata", "host", "kernel", "uptime"],
        fr: ReferenceText {
            title: "Identite systeme",
            summary: "Nom d'hote, OS, kernel, uptime, architecture et nombre de CPU.",
            beginner: "Base utile pour savoir exactement quelle machine on lit.",
            expert: "Conditionne l'interpretation correcte des autres metriques.",
        },
        en: ReferenceText {
            title: "System identity",
            summary: "Hostname, OS, kernel, uptime, architecture and CPU count.",
            beginner: "Useful baseline to know exactly which machine you are reading.",
            expert: "Shapes the correct interpretation of the other metrics.",
        },
    },
    ReferenceEntry {
        id: "derived.cpu_trend",
        category: "derived",
        panel: "cpu",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "cpu trend",
            "cpu trend percentiles",
            "p50",
            "p95",
            "trend percentiles",
        ],
        tags: &["derived", "cpu", "trend", "percentiles"],
        fr: ReferenceText {
            title: "Tendance CPU",
            summary: "Percentiles derives pour lisser la lecture CPU recente.",
            beginner: "Aide a voir si la charge est ponctuelle ou installee.",
            expert: "Une vue plus robuste qu'un instantane unique.",
        },
        en: ReferenceText {
            title: "CPU trend",
            summary: "Derived percentiles smoothing recent CPU behavior.",
            beginner: "Helps tell whether load is spiky or sustained.",
            expert: "A more robust view than a single instant snapshot.",
        },
    },
    ReferenceEntry {
        id: "derived.future",
        category: "derived",
        panel: "alerts",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "health index",
            "synthetic health indices",
            "anomaly detection",
            "correlation engine",
            "correlation engine os app",
            "correlation engine os ↔ app",
        ],
        tags: &["derived", "health", "anomaly", "correlation", "planned"],
        fr: ReferenceText {
            title: "Indices et intelligence derives",
            summary: "Indices de sante, anomalies et correlation, encore planifies.",
            beginner: "Ces aides doivent simplifier la lecture, pas la rendre opaque.",
            expert: "Ils doivent rester relies aux signaux bruts et explicables.",
        },
        en: ReferenceText {
            title: "Derived intelligence",
            summary: "Health indices, anomaly detection and correlation, still planned.",
            beginner: "These helpers should simplify reading, not make it opaque.",
            expert: "They must remain grounded in raw explainable signals.",
        },
    },
    ReferenceEntry {
        id: "infra.future",
        category: "infrastructure",
        panel: "linux",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &[
            "numa",
            "numa metrics",
            "ipc",
            "ipc monitoring",
            "security events",
            "ebpf",
            "ebpf option",
        ],
        tags: &["infrastructure", "numa", "ipc", "security", "ebpf", "planned"],
        fr: ReferenceText {
            title: "Signaux infrastructure avances",
            summary: "NUMA, IPC, securite et eBPF font partie de la surface future.",
            beginner: "Pas indispensables pour une V1 locale credible.",
            expert: "Ils doivent vivre dans la meme base de reference meme s'ils ne sont pas encore rendus.",
        },
        en: ReferenceText {
            title: "Advanced infrastructure signals",
            summary: "NUMA, IPC, security and eBPF belong to the future surface area.",
            beginner: "Not required for a credible local V1.",
            expert: "They should live in the same reference base even before they are rendered.",
        },
    },
    ReferenceEntry {
        id: "runtime.current",
        category: "runtime",
        panel: "alerts",
        status: MetricStatus::Implemented,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Beginner,
        aliases: &[
            "tui mode",
            "json export",
            "csv export",
            "prometheus text export",
            "record mode",
            "replay mode",
            "service install scaffolding",
        ],
        tags: &["runtime", "tui", "json", "csv", "prometheus", "record", "replay", "service"],
        fr: ReferenceText {
            title: "Capacites runtime actuelles",
            summary: "Regroupe les modes et exports deja disponibles dans Pulsar.",
            beginner: "Ce bloc couvre les fonctions que l'on peut deja utiliser au quotidien.",
            expert: "Le niveau de profondeur varie encore selon OS et selon la surface runtime concernee.",
        },
        en: ReferenceText {
            title: "Current runtime capabilities",
            summary: "Groups the modes and exports already available in Pulsar.",
            beginner: "This block covers the features already usable day to day.",
            expert: "Depth still varies by OS and by the runtime surface involved.",
        },
    },
    ReferenceEntry {
        id: "runtime.future",
        category: "runtime",
        panel: "alerts",
        status: MetricStatus::Planned,
        ui_visibility: UiVisibility::IndexedOnly,
        audience: Audience::Expert,
        aliases: &["multi-host / distributed mode", "enterprise controls"],
        tags: &["runtime", "distributed", "enterprise", "planned"],
        fr: ReferenceText {
            title: "Capacites runtime futures",
            summary: "Modes distribues et controles enterprise, encore hors scope courant.",
            beginner: "Pas necessaire pour la V1 locale actuelle.",
            expert: "Doit vivre dans la meme base de reference meme avant implementation.",
        },
        en: ReferenceText {
            title: "Future runtime capabilities",
            summary: "Distributed mode and enterprise controls, still outside current scope.",
            beginner: "Not required for the current local V1.",
            expert: "Should live in the same reference base even before implementation.",
        },
    },
];

pub fn catalog_views(locale: Locale) -> Vec<ReferenceEntryView> {
    CATALOG.iter().map(|entry| to_view(entry, locale)).collect()
}

pub fn search(query: &str, locale: Locale) -> Vec<SearchHit> {
    let normalized = normalize(query);
    let mut hits: Vec<SearchHit> = CATALOG
        .iter()
        .filter_map(|entry| {
            score_entry(entry, &normalized).map(|score| SearchHit {
                score,
                entry: to_view(entry, locale),
            })
        })
        .collect();

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.entry.title.cmp(b.entry.title))
    });
    hits
}

pub fn panel_matches_query(panel: &str, query: &str) -> bool {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return false;
    }

    CATALOG
        .iter()
        .any(|entry| entry.panel == panel && score_entry(entry, &normalized).is_some())
}

fn to_view(entry: &ReferenceEntry, locale: Locale) -> ReferenceEntryView {
    let text = match locale {
        Locale::Fr => entry.fr,
        Locale::En => entry.en,
    };

    ReferenceEntryView {
        id: entry.id,
        category: entry.category,
        panel: entry.panel,
        status: entry.status,
        ui_visibility: entry.ui_visibility,
        audience: entry.audience,
        title: text.title,
        summary: text.summary,
        beginner: text.beginner,
        expert: text.expert,
        aliases: entry.aliases,
        tags: entry.tags,
    }
}

fn score_entry(entry: &ReferenceEntry, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(1);
    }

    let mut score = 0;
    for candidate in search_terms(entry) {
        let normalized = normalize(candidate);
        if normalized == query {
            score = score.max(100);
        } else if normalized.contains(query) {
            score = score.max(60);
        } else if query
            .split_whitespace()
            .all(|part| normalized.contains(part))
        {
            score = score.max(30);
        }
    }

    if score == 0 {
        None
    } else {
        Some(score)
    }
}

fn search_terms(entry: &ReferenceEntry) -> Vec<&'static str> {
    let mut terms = vec![
        entry.id,
        entry.panel,
        entry.fr.title,
        entry.fr.summary,
        entry.en.title,
        entry.en.summary,
    ];
    terms.extend_from_slice(entry.aliases);
    terms.extend_from_slice(entry.tags);
    terms
}

fn normalize(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_alias_match() {
        let hits = search("latency", Locale::En);
        assert!(hits.iter().any(|hit| hit.entry.id == "disk.await"));
    }

    #[test]
    fn panel_query_match_is_detected() {
        assert!(panel_matches_query("memory", "swap"));
        assert!(!panel_matches_query("network", "swap"));
    }

    #[test]
    fn metrics_checklist_rows_are_covered_by_reference_catalog() {
        let checklist = include_str!("../docs/metrics-checklist.md");
        let missing: Vec<String> = checklist
            .lines()
            .filter_map(extract_checklist_label)
            .filter(|label| search(label, Locale::En).is_empty())
            .map(str::to_string)
            .collect();

        assert!(
            missing.is_empty(),
            "metrics checklist entries missing from reference catalog: {:?}",
            missing
        );
    }

    fn extract_checklist_label(line: &str) -> Option<&str> {
        if !line.starts_with("| [") {
            return None;
        }

        let mut cells = line.split('|').map(str::trim);
        let _leading = cells.next()?;
        let checklist = cells.next()?;
        let label = checklist
            .trim_start_matches("[x]")
            .trim_start_matches("[ ]")
            .trim();

        if label.is_empty() {
            None
        } else {
            Some(label)
        }
    }
}
