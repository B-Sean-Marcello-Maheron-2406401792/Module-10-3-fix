### Experiment 3.1: Original code
![img_1.png](img_1.png)
![img.png](img.png)

NOTE: Karena source code yang digunakan sudah tidak kompatible dengan versi rust sekarang, saya membuat sendiri code klien dan servernya.

### Experiment 3.2: Be Creative!

![img_2.png](img_2.png)

![img_3.png](img_3.png)

Untuk mewujudkan pembaruan kreatif dan penambahan fitur tersebut, saya melakukan modifikasi komprehensif yang berpusat pada dua berkas utama di sisi klien, yaitu index.html dan  
src/lib.rs. Pada berkas index.html, saya menyisipkan skrip pustaka ikon Lucide via CDN dan merombak blok CSS secara signifikan dengan menambahkan animasi flicker neon pada judul
utama, efek garis pemindai bergaya terminal retro, serta mendefinisikan tata letak (layout) khusus untuk halaman Gallery dan panel Emoji Picker agar dapat melayang (floating)    
dengan efek pijaran (glow) yang konsisten. Sementara itu, pada berkas logika inti src/lib.rs, perubahannya jauh lebih mendalam; saya memperbarui enum Screen untuk mendaftarkan   
rute halaman Galeri dan menambahkan enum pesan (Msg) baru seperti ToggleEmojiPicker dan AddEmoji untuk menangani interaksi. Di dalam struct App, saya menambahkan state
emoji_picker_open guna mengontrol visibilitas panel, memperluas daftar konstanta AVATARS, serta menciptakan konstanta EMOJIS baru yang memuat ratusan karakter emoji. Saya        
kemudian menyusun fungsi view_gallery dari awal untuk menampilkan koleksi ikon artistik, sekaligus memodifikasi fungsi view_login, view_about, dan view_chat untuk menanamkan     
antarmuka ikon Lucide, mengubah gaya bahasa antarmuka menjadi tema cyberpunk (seperti "transmit neural pulse"), dan memasang tombol toggle emoji beserta struktur panelnya di area
masukan pesan. Sebagai penyempurna, saya mengimplementasikan fungsi lifecycle rendered untuk mengeksekusi skrip JavaScript yang menggambar ikon Lucide ke dalam DOM secara        
dinamis, serta memperbarui logika fungsi update agar setiap emoji yang dipilih langsung dirangkai ke dalam teks masukan dan panel emoji otomatis menutup seketika setelah pesan   
berhasil ditransmisikan.

### BONUS:
Transformasi server chat-async berhasil dilakukan dengan mengubah perannya dari sekadar penyalur teks mentah menjadi infrastruktur backend yang cerdas, terstruktur, dan tangguh untuk melayani klien YewChat. Melalui integrasi pustaka serde dan serde_json, server kini memiliki kemampuan untuk menangani pesan berbasis objek secara native melalui struktur data ChatMessage. Server tidak lagi sekadar meneruskan data, melainkan berperan sebagai middleware aktif yang membedah setiap pesan masuk, melakukan deserialisasi, dan memperkayanya dengan metadata krusial seperti timestamp presisi dari chrono serta alamat IP asli pengirim. Dengan menyematkan informasi ini di sisi server sebelum pesan disiarkan kembali, integritas, akurasi, dan kronologi data di sisi klien menjadi terjamin sepenuhnya.

Secara teknis, langkah strategis berupa sinkronisasi port ke 9001 serta optimalisasi arsitektur event-driven berbasis tokio memastikan interoperabilitas yang mulus antara backend Rust dan aplikasi antarmuka visual berbasis WebAssembly. Pendekatan ini secara efektif membangun sebuah otoritas data yang terpusat; di mana tanggung jawab validasi dan pengayaan pesan tidak lagi didelegasikan ke klien, melainkan dikontrol secara ketat oleh server. Hal ini tidak hanya menyederhanakan logika di sisi frontend, tetapi juga menutup celah bagi pengguna untuk melakukan manipulasi atau pemalsuan informasi terkait waktu pengiriman maupun identitas asal jaringan mereka.

Dari sisi pengembangan, pilihan untuk tetap menggunakan Rust dibandingkan beralih ke ekosistem JavaScript yang lebih instan, terbukti sebagai keputusan arsitektural yang tepat. Meskipun JavaScript menawarkan kemudahan dalam manipulasi objek JSON, kelonggaran tipenya sering menjadi bumerang. Sebaliknya, sistem strict typing dan fitur pattern matching pada Rust memaksa penanganan skenario kegagalan secara eksplisit sejak tahap kompilasi. Jaminan keamanan ini secara drastis menekan risiko runtime error yang sering ditemui pada server JavaScript. Digabungkan dengan efisiensi memori yang luar biasa dan performa mentah dari runtime tokio, peladen ini kini bertransformasi menjadi infrastruktur tingkat produksi yang jauh lebih andal dan efisien dibandingkan solusi event-loop standar.