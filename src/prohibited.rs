use hashbrown::HashSet;

use crate::{dictionary::Dictionary, key::Key, keyboard::Keyboard, penalty::Penalty};

pub struct Prohibited(HashSet<Key>);

impl Prohibited {
    pub fn new() -> Prohibited {
        Prohibited(HashSet::new())
    }

    pub fn with_top_n_letter_pairs(dict: &Dictionary, top_n: usize) -> Prohibited {
        if dict.words().len() > 307_000 {
            let cache = "ai,st,ns,nt,io,fn,ds,ao,ey,eo,tw,bm,hw,nr,ae,bh,dt,ms,dr,rs,mw,fr,bw,dn,fs,mt,ei,hm,ft,rt,mn,lr,au,dm,ps,cs,ln,ls,lm,dl,gs,gn,lt,dg,iu,mp,ny,sy,gt,mr,et,ct,cw,bo,hn,er,sw,no,cm,ou,lp,as,hs,pt,hr,bt,dp,dk,bs,gl,my,cp,dw,bl,hl,fl,es,ch,cr,bp,fm,br,dy,cn,eh,de,cg,bc,ht,pr,is,ar,lw,el,hp,nw,eu,np,cd,bf,fw,bd,cf,fk,fh,ry,cl,ah,dh,op,or,bg,rv,pw,at,nv,rw,ac,kn,em,fp,ru,gr,ow,al,kt,km,oy,gm,ks,fg,ty,am,ad,ap,ep,bn,mo,gk,gh,ir,in,en,it,ce,kl,df,an,sv,mv,gp,hk,kr,kp,jm,di,wy,il,im,os,co,gw,ek,ot,eg,ci,fv,be,lo,lv,hy,ab,kv,do,tv,gy,dv,ef,ly,ip,ag,fy,bk,hi,ck,cv,bj,pv,ew,by,ay,kw,tx,bi,ho,cy,py,jp,su,af,iy,jt,fi,ak,dj,bv,js,jl,mu,rx,tu,nu,jr,nx,pu,sx,gi,go,cu,jn,vw,gj,gv,iv,aw,du,sz,dx,iw,lu,fo,av,dz,cj,fj,ex,hv,hu,hj,tz,ov,ax,lx,bu,ev,jw,px,ik,ky,gu,mx,cx,xy,ko,vy,uv,jk,fu,aj,bx,uw,ix,uy,jy,wx,ij,jv,gx,ox,jo,ej,ku,ux,vx,nz,mz,lz,hx,fx,kx,rz,bz,cz,ju,gz,hz,pz,jx,kz,qs,ez,fz,az,vz,yz,wz,gq,jz,dq,iz,nq,oz,e',cq,qr,aq,qt,mq,lq,pq,bq,iq,eq,uz,oq,a',fq,hq,kq,n',qw,qu,qv,xz,qy,jq,qx,r',l',qz,i',t',u',o',d',b',p',s',m',g',c',k',y',h',f',w',v',x',j',q',z'";
            let mut result = Prohibited::new();
            let pairs = cache.split(",").map(|pair| Key::new(pair)).take(top_n);
            result.add_many(pairs);
            result
        } else {
            let mut penalties = dict
                .alphabet()
                .subsets_of_size(2)
                .map(|pair| {
                    let k = Keyboard::with_keys(vec![pair]).fill_missing(dict.alphabet());
                    let p = k.penalty(&dict, Penalty::MAX);
                    (pair, p)
                })
                .collect::<Vec<(Key, Penalty)>>();
            penalties.sort_by(|a, b| {
                let (_a_key, a_penalty) = a;
                let (_b_key, b_penalty) = b;
                b_penalty.cmp(a_penalty)
            });
            let mut result = Prohibited::new();
            result.add_many(
                penalties
                    .into_iter()
                    .take(top_n)
                    .map(|(pair, _penalty)| pair),
            );
            result
        }
    }

    pub fn is_allowed(&self, other: Key) -> bool {
        !self.0.iter().any(|p| other.contains_all(&p))
    }

    pub fn add(&mut self, key: Key) {
        if key.is_empty() {
            panic!("Can not add an empty key to the list of prohibited keys.")
        }
        let subsets = self
            .0
            .iter()
            .filter(|i| key.contains_all(i) && key != **i)
            .map(|i| i.clone())
            .collect::<Vec<Key>>();
        let supersets = self
            .0
            .iter()
            .filter(|i| (**i).contains_all(&key) && key != **i)
            .map(|i| i.clone())
            .collect::<Vec<Key>>();
        for s in supersets {
            self.0.remove(&s);
        }
        if subsets.is_empty() {
            self.0.insert(key);
        }
    }

    pub fn add_many(&mut self, keys: impl Iterator<Item = Key>) {
        for k in keys {
            self.add(k);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn is_allowed() {
        let data = [
            ("ae,st", "ae", false),
            ("ae,st", "st", false),
            ("ae,st", "xy", true),
            ("ae,st", "a", true),
            ("ae,st", "s", true),
            ("ae,st", "t", true),
            ("ae,st", "aer", false),
            ("ae,st", "stu", false),
            ("ast", "ast", false),
            ("ast", "astx", false),
            ("ast", "as", true),
            ("ast", "str", true),
        ];
        for (prohibited, test_key, expect_is_key_allowed) in data {
            let mut p = Prohibited::new();
            p.add_many(prohibited.split(',').map(|pattern| Key::new(pattern)));
            let actual = p.is_allowed(Key::new(test_key));
            assert_eq!(
                actual, expect_is_key_allowed,
                "prohibited [{}] test [{}] expect_is_allowed {}",
                prohibited, test_key, expect_is_key_allowed
            );
        }
    }

    #[test]
    #[should_panic]
    fn if_add_empty_key_panic() {
        let mut p = Prohibited::new();
        p.add(Key::EMPTY);
    }

    #[test]
    fn duplicates_removed_when_add_smaller_key_first() {
        let mut p = Prohibited::new();
        p.add(Key::new("ab"));
        p.add(Key::new("abc"));
        assert_eq!(p.0.len(), 1);
        assert_eq!(Key::new("ab"), p.0.into_iter().next().unwrap());
    }

    #[test]
    fn duplicates_are_removed_when_add_bigger_key_first() {
        let mut p = Prohibited::new();
        p.add(Key::new("abc"));
        p.add(Key::new("ab"));
        assert_eq!(p.0.len(), 1);
        assert_eq!(Key::new("ab"), p.0.into_iter().next().unwrap());
    }
}
