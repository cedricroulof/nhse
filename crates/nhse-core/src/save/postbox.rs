//! Accessors for a player's `postbox.dat`.
use super::horizon::HorizonSave;
use crate::error::{Error, Result};
use crate::structs::mail::{self, Mail};
use crate::util::bytes::*;

impl HorizonSave {
    fn mail_offset(&self, slot: usize) -> usize {
        self.postbox_ofs.mail_list + slot * mail::SIZE
    }

    fn postbox_data(&self, p: usize) -> Result<&[u8]> {
        self.players.get(p).map(|x| x.postbox.data.as_slice()).ok_or(Error::OutOfRange { index: p, max: self.players.len() })
    }

    pub fn mails(&self, p: usize) -> Result<Vec<Mail>> {
        let d = self.postbox_data(p)?;
        Ok((0..mail::SLOTS).map(|i| Mail::read(d, self.mail_offset(i), i)).collect())
    }

    pub fn set_mail(&mut self, p: usize, m: &Mail) -> Result<()> {
        if m.slot >= mail::SLOTS {
            return Err(Error::OutOfRange { index: m.slot, max: mail::SLOTS });
        }
        let o = self.mail_offset(m.slot);
        let d = &mut self.players.get_mut(p).ok_or(Error::OutOfRange { index: p, max: 8 })?.postbox.data;
        m.write(d, o);
        Ok(())
    }

    pub fn clear_mail(&mut self, p: usize, slot: usize) -> Result<()> {
        let o = self.mail_offset(slot);
        let d = &mut self.players.get_mut(p).ok_or(Error::OutOfRange { index: p, max: 8 })?.postbox.data;
        Mail::clear(d, o);
        Ok(())
    }

    pub fn mail_raw(&self, p: usize, slot: usize) -> Result<Vec<u8>> {
        let o = self.mail_offset(slot);
        Ok(self.postbox_data(p)?[o..o + mail::SIZE].to_vec())
    }

    pub fn set_mail_raw(&mut self, p: usize, slot: usize, bytes: &[u8]) -> Result<()> {
        if bytes.len() != mail::SIZE {
            return Err(Error::Invalid(format!("mail must be {} bytes, got {}", mail::SIZE, bytes.len())));
        }
        let o = self.mail_offset(slot);
        let d = &mut self.players.get_mut(p).ok_or(Error::OutOfRange { index: p, max: 8 })?.postbox.data;
        d[o..o + mail::SIZE].copy_from_slice(bytes);
        Ok(())
    }

    pub fn postbox_next_id(&self, p: usize) -> Result<u32> {
        Ok(u32le(self.postbox_data(p)?, self.postbox_ofs.latest_unique_id))
    }

    pub fn set_postbox_next_id(&mut self, p: usize, v: u32) -> Result<()> {
        let o = self.postbox_ofs.latest_unique_id;
        let d = &mut self.players.get_mut(p).ok_or(Error::OutOfRange { index: p, max: 8 })?.postbox.data;
        put_u32(d, o, v);
        Ok(())
    }
}
