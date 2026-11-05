// Execute the mission-transition tail of 433ca0; observe only the music,
// snprintf and media-player leaves. No original code or resources are copied.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
public class RootV155CampaignStartFilmOracle extends GhidraScript {
 static final long S=0x7fff0000L,END=0x4344a1L,HALT=0x12345100L,SPR=0x12345200L;
 long read(EmulatorHelper e,long a,int n){byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 String stringAt(EmulatorHelper e,long a){StringBuilder b=new StringBuilder();for(int i=0;i<80;i++){int n=(int)read(e,a+i,1);if(n==0)return b.toString();if(n<32||n>126)throw new IllegalArgumentException("string "+Long.toHexString(a));b.append((char)n);}throw new IllegalArgumentException("string length");}
 public void run()throws Exception{
  EmulatorHelper e=new EmulatorHelper(currentProgram);
  int[] worlds={0,3,1,4,2};
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(long a:new long[]{END,HALT,SPR,0x431110,0x431820})e.setBreakpoint(toAddr(a));
   put(e,0x474220,4,HALT);put(e,0x474054,4,SPR);
   for(int mission=1;mission<=20;mission++)for(int before:new int[]{-1,0,1,2,3,4})
   for(int mp=0;mp<2;mp++)for(int media=0;media<3;media++){
    for(String reg:new String[]{"EAX","ECX","EDX","EBX","EDI","EBP","DF"})e.writeRegister(reg,0);
    e.writeRegister("ESI",1);e.writeRegister("ZF",mp==0?1:0);
    e.writeMemory(toAddr(S-128),new byte[512]);e.writeRegister("ESP",S);
    int world=worlds[(mission-1)/4],previous=mission==1?-1:worlds[(mission-2)/4];
    put(e,0x622c7a,2,mission);put(e,0x622dc4,1,mp);put(e,0x47705f,1,before);
    put(e,0x625eae+mission*240,1,world);put(e,0x625dbe+mission*240,1,previous);
    put(e,0x4d02f4,2,99);put(e,0x4d02f0,4,123);put(e,0x62d5d3,1,17);
    if(!e.run(toAddr(0x434305),null,monitor))throw new Exception(e.getLastError());
    String movie="";int played=0,halt=0,playMode=-1,playFlag=-1;
    for(int count=0;e.readRegister("EIP").longValue()!=END;count++){
     if(count>8)throw new Exception("call limit");
     long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
     if(pc==HALT){halt++;}
     else if(pc==SPR){
      String format=stringAt(e,read(e,sp+8,4));if(!format.endsWith("%d"))throw new Exception(format);
      String name=format.substring(0,format.length()-2)+(int)read(e,sp+12,4);
      e.writeMemory(toAddr(read(e,sp+4,4)),(name+"\0").getBytes(StandardCharsets.US_ASCII));e.writeRegister("EAX",name.length());
     }else if(pc==0x431110){
      movie=stringAt(e,e.readRegister("ECX").longValue());e.writeRegister("EAX",media==0?0:1);
     }else if(pc==0x431820){
      played++;playMode=e.readRegister("ECX").intValue();playFlag=e.readRegister("EDX").intValue();e.writeRegister("EAX",media==2?1:0);
     }else throw new Exception(Long.toHexString(pc));
     e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);
     if(!e.run(monitor))throw new Exception(e.getLastError());
    }
    out.printf("{\"mission\":%d,\"world\":%d,\"previous\":%d,\"before\":%d,\"mp\":%d,\"media\":%d,\"movie\":\"%s\",\"played\":%d,\"halt\":%d,\"playMode\":%d,\"playFlag\":%d,\"next\":%d,\"movieMode\":%d,\"after\":%d}%n",mission,world,previous,before,mp,media,movie,played,halt,playMode,playFlag,e.readRegister("EAX").intValue(),read(e,0x4d02f4,2),(byte)read(e,0x47705f,1));
    out.flush();
   }
  }finally{e.dispose();}
 }
}
