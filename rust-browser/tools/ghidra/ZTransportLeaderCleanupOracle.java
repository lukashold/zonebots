import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;

// Actual59d80 ->52500, all256 transport flag bytes and active/inactive members.
// Leader role-1 executes the actual57790 gate with group58=0.
public class ZTransportLeaderCleanupOracle extends ZCoupledFollowerOracle {
 public void run()throws Exception {
  byte[] trig=Files.readAllBytes(Path.of(System.getenv().getOrDefault("Z_GAME_DIR","C:/GOG Games/Z"),"SINCOS.DAT"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])) {
   for(int s=0;s<512;s++) {
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try {
     setup(e,s%128,false,false,trig);long u=U+192,q=0x5f8428L;
     for(int r=0;r<21;r++)put(e,R+19+r,1,255);
     put(e,R+19,1,1);put(e,u+0x31,1,(s<256?4:0)|32);
     put(e,u+0x8a,1,s%256);put(e,u+0x5e,1,255);put(e,R+58,1,0);
     put(e,u+0x48,2,s%32767);put(e,u+0x4a,2,123);put(e,u+0x4c,2,456);put(e,u+0x4e,2,789);
     put(e,u+0x14,4,P);put(e,P,2,900);put(e,P+2,2,800);put(e,P+4,4,0);
     put(e,u+0xc,4,q);put(e,q,2,700);put(e,q+2,2,600);put(e,q+4,4,0);
     put(e,0x62dc08+23,1,255);
     call(e,0x459d80,u);
     out.printf("{\"scenario\":%d,\"values\":[%s],\"flags8a\":%d,\"secondary\":%d,\"primaryNext\":%d,\"secondaryNext\":%d}%n",s,fields(e),get(e,u+0x8a,1),get(e,u+0xc,4),get(e,P+4,4),get(e,q+4,4));out.flush();
    }finally{e.dispose();}
   }
  }
 }
}
