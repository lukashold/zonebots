// Reproduce starting-position arithmetic using a locally supplied executable
// and local campaign resources. Output contains numerical coordinates only.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
import java.util.*;
public class ZNativeStartingPositionsOracle extends GhidraScript {
 static final long RET=0x12345000L,STACK=0x7fff0000L,OUT=0x1100000L;
 long read(EmulatorHelper e,long a,int n) {
  byte[] b=e.readMemory(toAddr(a),n);long v=0;
  for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;
 }
 void put(EmulatorHelper e,long a,int n,long v)throws Exception {e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a,long ecx,long edx)throws Exception {
  e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);
  e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());
  if(e.readRegister("EIP").longValue()!=RET)throw new Exception("Unexpected stop");
 }
 String ascii(byte[] b,int p,int n) {
  int j=p;while(j<p+n&&b[j]!=0)j++;return new String(b,p,j-p,java.nio.charset.StandardCharsets.ISO_8859_1);
 }
 public void run()throws Exception {
  Path root=Paths.get(System.getenv().getOrDefault("Z_GAME_DIR","C:/GOG Games/Z"));byte[] robot=Files.readAllBytes(root.resolve("robots.dat"));
  byte[] levels=Files.readAllBytes(root.resolve("levels.dat"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])) {
   for(int mission=1;mission<=20;mission++) {
    String mapName=String.format("level%02d.map",mission);int li=-1;
    for(int i=0;i<levels.length/240;i++)if(ascii(levels,i*240+20,13).equalsIgnoreCase(mapName)){li=i;break;}
    if(li<0)throw new Exception("Missing mission "+mission);
    byte[] cp=Files.readAllBytes(root.resolve(String.format("CPUPLR%02d.DAT",mission)));
    byte[] map=Files.readAllBytes(root.resolve(String.format("LEVEL%02d.MAP",mission)));
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try {
     e.writeMemory(toAddr(0x615498),robot);e.writeMemory(toAddr(0x625e40),levels);e.writeMemory(toAddr(0x5c3f0c),cp);
     e.writeMemory(toAddr(0x586aaa),Arrays.copyOfRange(map,0x2791,0x6791));
     e.writeMemory(toAddr(0x58aaaa),Arrays.copyOfRange(map,0x6791,0xa791));
     byte[] roster=new byte[240];Arrays.fill(roster,(byte)255);System.arraycopy(levels,li*240+0x55,roster,0,12);
     e.writeMemory(toAddr(0x632198),roster);put(e,0x622c7a,2,li);put(e,0x622dc4,1,0);
     invoke(e,0x417040,0,0);invoke(e,0x424bd0,0,0);invoke(e,0x45f860,0,0);
     for(int team=0;team<2;team++) {
      int bx=(short)read(e,team==0?0x622d58:0x622d54,2),by=(short)read(e,team==0?0x622d5a:0x622d56,2);
      put(e,0x632380,2,bx);put(e,0x63237c,2,by);put(e,0x584962,1,team*120);put(e,0x584963,1,0);
      put(e,0x4a8bfc,2,0);put(e,0x4a8c04,2,4);invoke(e,0x41aca0,team+1,0);
      for(int slot=0;slot<120;slot++) {
       int group=(int)read(e,0x632198+team*120+slot,1);if(group==255)continue;
       int cls=(int)read(e,0x632288+team*120+slot,1);boolean fixed=cls==2||cls==4||cls==5||cls==6;
       int counter=-1,label=-1,mapped=-1;
       if(fixed) {
        counter=(short)read(e,team==0?0x4a8bfc:0x4a8c04,2);
        label=(int)read(e,(team==0?0x4982c8:0x4981a8)+li*7+counter,1);mapped=(int)read(e,0x498123+label,1);
       } else put(e,0x4a8c08,4,read(e,0x4a8c08,4)+4);
       put(e,0x5849d6,1,-1-slot);put(e,0x4a8bf8,2,team*120+slot);invoke(e,0x41aff0,OUT,OUT+2);
       int x=(short)read(e,OUT,2),y=(short)read(e,OUT+2,2);
       int cpx=team==1?(short)read(e,0x5c46a0+slot*10,2):-1,cpy=team==1?(short)read(e,0x5c46a2+slot*10,2):-1;
       out.println("{\"mission\":"+mission+",\"levelIndex\":"+li+",\"team\":"+team+",\"slot\":"+slot+",\"group\":"+group+",\"class\":"+cls+",\"counter\":"+counter+",\"label\":"+label+",\"mapped\":"+mapped+",\"baseX\":"+bx+",\"baseY\":"+by+",\"x\":"+x+",\"y\":"+y+",\"cpX\":"+cpx+",\"cpY\":"+cpy+"}");
      }
     }
    } finally {e.dispose();}
   }
  }
 }
}
